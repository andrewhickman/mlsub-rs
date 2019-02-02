use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::Hash;

use im::Vector;

use crate::auto::{flow, Automaton, State, StateId, Symbol, FlowSet};
use crate::polar;
use crate::{Polarity, TypeSystem};

pub trait Build<T: TypeSystem, V>: Sized {
    fn constructor(&self) -> T::Constructor;
    fn visit_transitions<'a, F>(&'a self, f: F)
    where
        V: 'a,
        F: FnMut(T::Symbol, &'a polar::Ty<Self, V>);
}

pub struct Builder<'a, T, V> 
where
    T: TypeSystem,
    V: Eq + Hash + Clone,
{
    auto: &'a mut Automaton<T>,
    vars: HashMap<StateId, (Polarity, Vec<V>)>,
}

impl<'a, T: TypeSystem> Automaton<T> {
    pub fn builder<V: Eq + Hash + Clone>(&'a mut self) -> Builder<T, V> {
        Builder {
            auto: self,
            vars: HashMap::new(),
        }
    }

    /// Build an empty state, representing the bottom and top types for positive and negative
    /// polarities respectively.
    pub fn build_empty(&mut self, pol: Polarity) -> StateId {
        self.add(State::new(pol))
    }

    /// Build an state representing the join or meet of some states for positive and negative
    /// polarities respectively.
    pub fn build_add<I>(&mut self, pol: Polarity, states: I) -> StateId 
    where
        I: IntoIterator<Item = StateId>
    {
        unimplemented!()
    }

    /// Create a type variable representing data flow from negative to positive states.
    pub(crate) fn build_var(&mut self) -> flow::Pair {
        let pair = flow::Pair {
            neg: self.build_empty(Polarity::Neg),
            pos: self.build_empty(Polarity::Pos),
        };
        self.add_flow(pair);
        pair
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

                self.merge(pol, at, expr);
            }
            polar::Ty::BoundVar(_) => unreachable!(),
            polar::Ty::Add(l, r) => {
                let l = self.build_polar_closure(pol, true, l, stack, recs);
                let r = self.build_polar_closure(pol, true, r, stack, recs);
                self.merge(pol, at, l);
                self.merge(pol, at, r);
            }
            polar::Ty::UnboundVar(var) => {
                self.vars.insert(at, (pol, vec![var.clone()]));
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
                        self.build_polar_closure(pol * symbol.polarity(), false, ty, stack, recs);
                    self.auto.index_mut(at).trans.add(symbol, id);
                });
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

    fn merge(&mut self, pol: Polarity, target: StateId, source: StateId) {
        match pol {
            Polarity::Pos => self.auto.merge_pos(target, source),
            Polarity::Neg => self.auto.merge_neg(target, source),
        }

        if let Some((_, vars)) = self.vars.get(&source).cloned() {
            self.vars
                .entry(target)
                .or_insert((pol, vec![]))
                .1
                .extend(vars);
        }
    }
}

impl<'a, T, V> Drop for Builder<'a, T, V>
where
    T: TypeSystem,
    V: Eq + Hash + Clone,
{
    fn drop(&mut self) {
        let mut map: HashMap<V, (FlowSet, FlowSet)> = HashMap::new();

        for (&id, (pol, vars)) in &self.vars {
            for var in vars {
                let (negs, poss) = map.entry(var.clone()).or_default();
                match pol {
                    Polarity::Pos => poss.add(id),
                    Polarity::Neg => negs.add(id),
                }
            }
        }

        for (negs, poss) in map.values() {
            for id in negs.iter() {
                self.auto.index_mut(id).flow.union(poss);
            }
            for id in poss.iter() {
                self.auto.index_mut(id).flow.union(negs);
            }
        }
    }
}
