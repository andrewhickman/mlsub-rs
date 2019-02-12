mod set;

pub use self::set::{StateSet, StateSetIter};

use std::ops::Range;

use crate::auto::{Automaton, ConstructorSet, FlowSet};
use crate::{Constructor, Polarity};

#[derive(Debug)]
pub struct State<C: Constructor> {
    #[cfg(debug_assertions)]
    pub(crate) pol: Polarity,
    pub(crate) cons: ConstructorSet<C>,
    pub(crate) flow: FlowSet,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StateId(usize);

#[derive(Debug, Clone)]
pub struct StateRange(Range<usize>);

impl<C: Constructor> State<C> {
    pub(crate) fn new(_pol: Polarity) -> Self {
        State {
            #[cfg(debug_assertions)]
            pol: _pol,
            cons: ConstructorSet::default(),
            flow: FlowSet::default(),
        }
    }
}

impl<C: Constructor> Clone for State<C> {
    fn clone(&self) -> Self {
        State {
            #[cfg(debug_assertions)]
            pol: self.pol,
            cons: self.cons.clone(),
            flow: self.flow.clone(),
        }
    }
}

impl<C: Constructor> Automaton<C> {
    pub(crate) fn next(&mut self) -> StateId {
        StateId(self.states.len())
    }

    pub(crate) fn add(&mut self, state: State<C>) -> StateId {
        let id = self.next();
        self.states.push(state);
        id
    }

    pub(crate) fn index(&self, StateId(id): StateId) -> &State<C> {
        &self.states[id]
    }

    pub(crate) fn index_mut(&mut self, StateId(id): StateId) -> &mut State<C> {
        &mut self.states[id]
    }

    pub(crate) fn index_mut2(
        &mut self,
        StateId(i): StateId,
        StateId(j): StateId,
    ) -> (&mut State<C>, &mut State<C>) {
        debug_assert_ne!(i, j);
        if i < j {
            let (l, r) = self.states.split_at_mut(j);
            (&mut l[i], &mut r[0])
        } else {
            let (l, r) = self.states.split_at_mut(i);
            (&mut r[0], &mut l[j])
        }
    }

    pub(crate) fn range_from(&mut self, StateId(start): StateId) -> StateRange {
        StateRange(start..self.states.len())
    }

    pub(crate) fn enumerate(&self) -> impl Iterator<Item = (StateId, &State<C>)> {
        self.states
            .iter()
            .enumerate()
            .map(|(id, st)| (StateId(id), st))
    }
}

impl Iterator for StateRange {
    type Item = StateId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(StateId)
    }
}
