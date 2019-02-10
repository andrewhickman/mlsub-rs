#[cfg(test)]
mod polar;

#[cfg(test)]
pub(crate) use self::polar::{Build, Builder};

use std::borrow::Cow;

use crate::auto::{flow, Automaton, State, StateId};
use crate::{Polarity, Constructor};

impl<'a, C: Constructor> Automaton<C> {
    /// Build an empty state, representing the bottom and top types for positive and negative
    /// polarities respectively.
    pub fn build_empty(&mut self, pol: Polarity) -> StateId {
        self.add(State::new(pol))
    }

    /// Build an state representing the join or meet of some states for positive and negative
    /// polarities respectively.
    pub fn build_add<I>(&mut self, pol: Polarity, states: I) -> StateId
    where
        I: IntoIterator<Item = StateId>,
    {
        let id = self.build_empty(pol);
        self.build_add_at(pol, id, states);
        id
    }

    fn build_add_at<I>(&mut self, pol: Polarity, at: StateId, states: I)
    where
        I: IntoIterator<Item = StateId>,
    {
        for source in states {
            self.merge(pol, at, source);
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

    pub fn build_constructed(&mut self, pol: Polarity, con: C) -> StateId {
        let at = self.build_empty(pol);
        self.build_constructed_at(pol, at, con);
        at
    }

    fn build_constructed_at(&mut self, pol: Polarity, at: StateId, con: C) {
        self.index_mut(at).cons.add(pol, Cow::Owned(con));
    }
}
