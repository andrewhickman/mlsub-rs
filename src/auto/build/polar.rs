use std::collections::HashMap;
use std::hash::Hash;

use im::Vector;

use crate::auto::{flow, Automaton, StateId, StateSet};
use crate::polar;
use crate::{Constructor, Label, Polarity};

pub trait Build<C: Constructor, V>: Sized {
    fn map<'a, F>(&'a self, mapper: F) -> C
    where
        V: 'a,
        F: FnMut(C::Label, &'a polar::Ty<Self, V>) -> StateSet;
}

pub struct Builder<'a, C, V>
where
    C: Constructor,
    V: Eq + Hash + Clone,
{
    auto: &'a mut Automaton<C>,
    vars: HashMap<V, flow::Pair>,
}

impl<'a, C: Constructor> Automaton<C> {
    pub fn builder<V: Eq + Hash + Clone>(&'a mut self) -> Builder<C, V> {
        Builder {
            auto: self,
            vars: HashMap::new(),
        }
    }
}

impl<'a, C, V> Builder<'a, C, V>
where
    C: Constructor,
    V: Eq + Hash + Clone,
{
    pub fn build_polar<B>(&mut self, pol: Polarity, ty: &polar::Ty<B, V>) -> StateId
    where
        B: Build<C, V>,
    {
        let at = self.auto.build_empty(pol);
        let mut stack = vec![(pol, at, ty, Vector::new())];
        while let Some((pol, at, ty, mut recs)) = stack.pop() {
            self.build_polar_closure_at(pol, at, ty, &mut stack, &mut recs);
        }
        at
    }

    fn build_polar_closure_at<'b, B>(
        &mut self,
        pol: Polarity,
        at: StateId,
        ty: &'b polar::Ty<B, V>,
        stack: &mut Vec<(Polarity, StateId, &'b polar::Ty<B, V>, Vector<StateId>)>,
        recs: &mut Vector<StateId>,
    ) where
        B: Build<C, V>,
    {
        // TODO produce less garbage states

        #[cfg(debug_assertions)]
        debug_assert_eq!(self.auto.index(at).pol, pol);

        match ty {
            polar::Ty::Recursive(inner) => {
                recs.push_front(at);
                let expr = self.build_polar_closure(pol, true, inner, stack, recs);
                recs.pop_front();

                self.auto.merge(pol, at, expr);
            }
            polar::Ty::BoundVar(_) => unreachable!(),
            polar::Ty::Add(l, r) => {
                let l = self.build_polar_closure(pol, true, l, stack, recs);
                let r = self.build_polar_closure(pol, true, r, stack, recs);

                self.auto.build_add_at(pol, at, [l, r].iter().cloned());
            }
            polar::Ty::UnboundVar(var) => {
                let Builder { vars, auto } = self;
                let pair = vars.entry(var.clone()).or_insert_with(|| auto.build_var());
                self.auto.merge_flow(pol, at, pair.get(pol));
            }
            polar::Ty::Zero => (),
            polar::Ty::Constructed(c) => {
                let con = c.map(|label, ty| {
                    StateSet::Singleton(self.build_polar_closure(
                        pol * label.polarity(),
                        false,
                        ty,
                        stack,
                        recs,
                    ))
                });
                self.auto.build_constructed_at(pol, at, con);
            }
        }
    }

    fn build_polar_closure<'b, B>(
        &mut self,
        pol: Polarity,
        epsilon: bool,
        ty: &'b polar::Ty<B, V>,
        stack: &mut Vec<(Polarity, StateId, &'b polar::Ty<B, V>, Vector<StateId>)>,
        recs: &mut Vector<StateId>,
    ) -> StateId
    where
        B: Build<C, V>,
    {
        if let polar::Ty::BoundVar(idx) = *ty {
            recs[idx]
        } else {
            let id = self.auto.build_empty(pol);
            if epsilon {
                self.build_polar_closure_at(pol, id, ty, stack, recs);
            } else {
                stack.push((pol, id, ty, recs.clone()));
            }
            id
        }
    }
}
