use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::Hash;

use im::Vector;

use crate::auto::{flow, Automaton, State, StateId, Symbol};
use crate::polar;
use crate::{Polarity, TypeSystem};

pub trait Build<T: TypeSystem, V>: Sized {
    fn constructor(&self) -> T::Constructor;
    fn visit_transitions<'a, F>(&'a self, f: F)
    where
        V: 'a,
        F: FnMut(T::Symbol, &'a polar::Ty<Self, V>);
}

pub struct Builder<T: TypeSystem, V> {
    auto: Automaton<T>,
    vars: HashMap<V, (Vec<StateId>, Vec<StateId>)>,
}

impl<T: TypeSystem> Automaton<T> {
    pub fn builder<V: Eq + Hash>() -> Builder<T, V> {
        Builder {
            auto: Automaton::new(),
            vars: HashMap::new(),
        }
    }

    fn merge(&mut self, pol: Polarity, target: StateId, source: StateId) {
        match pol {
            Polarity::Pos => self.merge_pos(target, source),
            Polarity::Neg => self.merge_neg(target, source),
        }
    }
}

impl<T, V> Builder<T, V>
where
    T: TypeSystem,
    V: Eq + Hash + Clone,
{
    pub fn build_polar<C>(&mut self, pol: Polarity, ty: &polar::Ty<C, V>) -> StateId
    where
        C: Build<T, V>,
    {
        let at = self.build_empty(pol);
        let mut stack = vec![(pol, at, ty, Vector::new())];
        while let Some((pol, at, ty, mut recs)) = stack.pop() {
            self.build_polar_closure_at(pol, at, ty, &mut stack, &mut recs);
        }
        at
    }

    fn build_polar_closure_at<'a, C>(
        &mut self,
        pol: Polarity,
        at: StateId,
        ty: &'a polar::Ty<C, V>,
        stack: &mut Vec<(Polarity, StateId, &'a polar::Ty<C, V>, Vector<StateId>)>,
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
                let expr = self.build_polar_closure(pol, false, inner, stack, recs);
                recs.pop_front();

                self.auto.merge(pol, at, expr);
            }
            polar::Ty::BoundVar(_) => unreachable!(),
            polar::Ty::Add(l, r) => {
                let l = self.build_polar_closure(pol, false, l, stack, recs);
                self.auto.merge(pol, at, l);

                let r = self.build_polar_closure(pol, false, r, stack, recs);
                self.auto.merge(pol, at, r);
            }
            polar::Ty::UnboundVar(var) => {
                let &mut (ref mut negs, ref mut poss) = self.vars.entry(var.clone()).or_default();
                match pol {
                    Polarity::Neg => {
                        negs.push(at);
                        for &pos in &*poss {
                            self.auto.add_flow(flow::Pair { pos, neg: at });
                        }
                    }
                    Polarity::Pos => {
                        poss.push(at);
                        for &neg in &*negs {
                            self.auto.add_flow(flow::Pair { neg, pos: at });
                        }
                    }
                };
            }
            polar::Ty::Zero => (),
            polar::Ty::Constructed(c) => {
                let con = Cow::Owned(c.constructor());
                match pol {
                    Polarity::Pos => self.auto.index_mut(at).cons.add_pos(con),
                    Polarity::Neg => self.auto.index_mut(at).cons.add_neg(con),
                };

                c.visit_transitions(|symbol, ty| {
                    let id =
                        self.build_polar_closure(pol * symbol.polarity(), true, ty, stack, recs);
                    self.auto.index_mut(at).trans.add(symbol, id);
                });
            }
        }
    }

    fn build_polar_closure<'a, C>(
        &mut self,
        pol: Polarity,
        out: bool,
        ty: &'a polar::Ty<C, V>,
        stack: &mut Vec<(Polarity, StateId, &'a polar::Ty<C, V>, Vector<StateId>)>,
        recs: &mut Vector<StateId>,
    ) -> StateId
    where
        C: Build<T, V>,
    {
        if let polar::Ty::BoundVar(idx) = *ty {
            recs[idx]
        } else {
            let id = self.build_empty(pol);
            if out {
                stack.push((pol, id, ty, recs.clone()));
            } else {
                self.build_polar_closure_at(pol, id, ty, stack, recs);
            }
            id
        }
    }

    pub fn build(self) -> Automaton<T> {
        self.auto
    }

    /// Build an empty state, representing the bottom and top types for positive and negative
    /// polarities respectively.
    pub(crate) fn build_empty(&mut self, pol: Polarity) -> StateId {
        self.auto.add(State::new(pol))
    }

    // pub(crate) fn build_flow<N, P>(&mut self, neg: N, pos: P)
    // where
    //     N: IntoIterator<Item = StateId>,
    //     N::IntoIter: Clone,
    //     P: IntoIterator<Item = StateId>,
    //     P::IntoIter: Clone,
    // {
    //     for (neg, pos) in iproduct!(neg, pos) {
    //         self.add_flow(flow::Pair { neg, pos });
    //     }
    // }

    // /// Create a type variable representing data flow from negative to positive states.
    // pub(crate) fn build_var(&mut self) -> flow::Pair {
    //     let pair = flow::Pair {
    //         neg: self.build_empty(Polarity::Neg),
    //         pos: self.build_empty(Polarity::Pos),
    //     };
    //     self.add_flow(pair);
    //     pair
    // }
}
