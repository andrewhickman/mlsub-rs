use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Deref;

use itertools::iproduct;

use crate::auto::{flow, Automaton, State, StateId, Symbol, Transition};
use crate::polar;
use crate::{Polarity, TypeSystem};

pub trait Build<T: TypeSystem, V> {
    fn build(&self, builder: &mut Builder<T, V>, pol: Polarity) -> StateId;
}

pub struct Builder<T: TypeSystem, V> {
    auto: Automaton<T>,
    recs: Vec<StateId>,
    vars: HashMap<V, (Vec<StateId>, Vec<StateId>)>,
}

impl<T: TypeSystem> Automaton<T> {
    pub fn builder<V: Eq + Hash>() -> Builder<T, V> {
        Builder {
            auto: Automaton::new(),
            recs: Vec::new(),
            vars: HashMap::new(),
        }
    }
}

impl<T, V> Builder<T, V>
where
    T: TypeSystem,
    V: Eq + Hash + Clone,
{
    pub fn build_constructor(&mut self, pol: Polarity, con: T::Constructor) -> StateId {
        let at = self.build_empty(pol);
        match pol {
            Polarity::Pos => self.auto.index_mut(at).cons.add_pos(Cow::Owned(con)),
            Polarity::Neg => self.auto.index_mut(at).cons.add_neg(Cow::Owned(con)),
        }
        at
    }

    pub fn build_transition<C>(&mut self, pol: Polarity, at: StateId, symbol: T::Symbol, con: C)
    where
        C: Build<T, V>,
    {
        #[cfg(debug_assertions)]
        debug_assert_eq!(self.auto.index(at).pol, pol);

        let id = con.build(self, pol * symbol.polarity());
        self.auto
            .index_mut(at)
            .trans
            .add(Transition::new(symbol, id));
    }

    pub fn build_transitions<C, I>(&mut self, pol: Polarity, at: StateId, trans: I)
    where
        C: Build<T, V>,
        I: IntoIterator<Item = (T::Symbol, C)>,
    {
        #[cfg(debug_assertions)]
        debug_assert_eq!(self.auto.index(at).pol, pol);

        for (symbol, con) in trans {
            self.build_transition(pol, at, symbol, con);
        }
    }

    pub fn build_polar<C>(&mut self, pol: Polarity, ty: &polar::Ty<C, V>) -> StateId
    where
        C: Build<T, V>,
    {
        // TODO produce less garbage states
        match ty {
            polar::Ty::Recursive(inner) => {
                let bind = self.build_empty(pol);

                self.recs.push(bind);
                let expr = self.build_polar(pol, inner);
                self.recs.pop();

                self.auto.merge(pol, bind, expr);
                bind
            }
            polar::Ty::BoundVar(rx) => {
                let ix = self.recs.len() - 1 - rx;
                self.recs[ix]
            }
            polar::Ty::Add(l, r) => {
                let union = self.build_empty(pol);

                let l = self.build_polar(pol, l);
                self.auto.merge(pol, union, l);

                let r = self.build_polar(pol, r);
                self.auto.merge(pol, union, r);

                union
            }
            polar::Ty::UnboundVar(var) => {
                let id = self.build_empty(pol);

                let &mut (ref mut neg, ref mut pos) = self.vars.entry(var.clone()).or_default();
                match pol {
                    Polarity::Neg => neg.push(id),
                    Polarity::Pos => pos.push(id),
                };

                id
            }
            polar::Ty::Zero => self.build_empty(pol),
            polar::Ty::Constructed(c) => c.build(self, pol),
        }
    }

    pub fn finish(mut self) -> Automaton<T> {
        debug_assert_eq!(self.recs.len(), 0);

        for (_, (negs, poss)) in self.vars {
            for (&neg, &pos) in iproduct!(&negs, &poss) {
                self.auto.add_flow(flow::Pair { neg, pos });
            }
        }

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

impl<T, D, V> Build<T, V> for D
where
    T: TypeSystem,
    D: Deref,
    D::Target: Build<T, V>,
{
    fn build(&self, builder: &mut Builder<T, V>, pol: Polarity) -> StateId {
        self.deref().build(builder, pol)
    }
}

impl<T, C, V> Build<T, V> for polar::Ty<C, V>
where
    T: TypeSystem,
    V: Eq + Hash + Clone,
    C: Build<T, V>,
{
    fn build(&self, builder: &mut Builder<T, V>, pol: Polarity) -> StateId {
        builder.build_polar(pol, self)
    }
}
