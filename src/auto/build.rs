use std::borrow::Cow;

use itertools::iproduct;

use crate::auto::{Automaton, State, StateId};
use crate::trans::{Symbol, Transition};
use crate::{flow, Polarity, TypeSystem};

impl<T: TypeSystem> Automaton<T> {
    /// Build an empty state, representing the bottom and top types for positive and negative
    /// polarities respectively.
    pub fn build_empty(&mut self, pol: Polarity) -> StateId {
        self.add(State::new(pol))
    }

    pub fn build_type<I>(&mut self, pol: Polarity, at: StateId, con: T::Constructor, trans: I)
    where
        I: IntoIterator<Item = (T::Symbol, StateId)>,
    {
        #[cfg(debug_assertions)]
        debug_assert_eq!(self.index(at).pol, pol);

        match pol {
            Polarity::Pos => self.index_mut(at).cons.add_pos(Cow::Owned(con)),
            Polarity::Neg => self.index_mut(at).cons.add_neg(Cow::Owned(con)),
        }
        for (symbol, id) in trans {
            #[cfg(debug_assertions)]
            debug_assert_eq!(self.index(id).pol, pol * symbol.polarity());

            self.index_mut(at).trans.add(Transition::new(symbol, id));
        }
    }

    pub fn build_flow<N, P>(&mut self, neg: N, pos: P)
    where
        N: IntoIterator<Item = StateId>,
        N::IntoIter: Clone,
        P: IntoIterator<Item = StateId>,
        P::IntoIter: Clone,
    {
        for (neg, pos) in iproduct!(neg, pos) {
            self.add_flow(flow::Pair { neg, pos });
        }
    }

    /// Create a type variable representing data flow from negative to positive states.
    pub fn build_var(&mut self) -> flow::Pair {
        let pair = flow::Pair {
            neg: self.build_empty(Polarity::Neg),
            pos: self.build_empty(Polarity::Pos),
        };
        self.add_flow(pair);
        pair
    }
}
