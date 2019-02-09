use std::collections::HashMap;
use std::hash::Hash;

use im::Vector;

use crate::auto::{flow, Automaton, StateId, StateSet};
use crate::polar;
use crate::{Label, Polarity, TypeSystem};

pub trait Build<T: TypeSystem, V>: Sized {
    fn map<'a, F>(&'a self, mapper: F) -> T::Constructor
    where
        V: 'a,
        F: FnMut(T::Label, &'a polar::Ty<Self, V>) -> StateSet;
}

pub struct Builder<'a, T, V>
where
    T: TypeSystem,
    V: Eq + Hash + Clone,
{
    auto: &'a mut Automaton<T>,
    vars: HashMap<V, flow::Pair>,
}

impl<'a, T: TypeSystem> Automaton<T> {
    pub fn builder<V: Eq + Hash + Clone>(&'a mut self) -> Builder<T, V> {
        Builder {
            auto: self,
            vars: HashMap::new(),
        }
    }
}

impl<'a, T, V> Builder<'a, T, V>
where
    T: TypeSystem,
    V: Eq + Hash + Clone,
{
    pub fn build_polar<C>(&mut self, pol: Polarity, ty: &polar::Ty<C, V>) -> StateId
    where
        C: Build<T, V>,
    {
        let at = self.auto.build_empty(pol);
        let mut stack = vec![(pol, at, ty, Vector::new())];
        while let Some((pol, at, ty, mut recs)) = stack.pop() {
            self.build_polar_closure_at(pol, at, ty, &mut stack, &mut recs);
        }
        at
    }

    fn build_polar_closure_at<'b, C>(
        &mut self,
        pol: Polarity,
        at: StateId,
        ty: &'b polar::Ty<C, V>,
        stack: &mut Vec<(Polarity, StateId, &'b polar::Ty<C, V>, Vector<StateId>)>,
        recs: &mut Vector<StateId>,
    ) where
        C: Build<T, V>,
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

    fn build_polar_closure<'b, C>(
        &mut self,
        pol: Polarity,
        epsilon: bool,
        ty: &'b polar::Ty<C, V>,
        stack: &mut Vec<(Polarity, StateId, &'b polar::Ty<C, V>, Vector<StateId>)>,
        recs: &mut Vector<StateId>,
    ) -> StateId
    where
        C: Build<T, V>,
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

    pub fn finish(self) {
        drop(self)
    }
}
