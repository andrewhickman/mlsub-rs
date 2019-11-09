mod set;

pub use self::set::{StateSet, StateSetIter};

use std::ops::{Index, IndexMut, Range};

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

impl StateId {
    pub(crate) fn shift(self, offset: usize) -> Self {
        StateId(self.0 + offset)
    }
}

impl<C: Constructor> State<C> {
    pub(crate) fn new(_pol: Polarity) -> Self {
        State {
            #[cfg(debug_assertions)]
            pol: _pol,
            cons: ConstructorSet::default(),
            flow: FlowSet::default(),
        }
    }

    pub fn constructors(&self) -> &ConstructorSet<C> {
        &self.cons
    }

    fn shift(self, offset: usize) -> Self {
        State {
            #[cfg(debug_assertions)]
            pol: self.pol,
            cons: self.cons.shift(offset),
            flow: self.flow.shift(offset),
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

    pub(crate) fn append(&mut self, other: &mut Self) {
        let offset = self.states.len();
        self.states
            .extend(other.states.drain(..).map(|state| state.shift(offset)))
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

impl<C: Constructor> Index<StateId> for Automaton<C> {
    type Output = State<C>;

    fn index(&self, StateId(id): StateId) -> &Self::Output {
        self.states.index(id)
    }
}

impl<C: Constructor> IndexMut<StateId> for Automaton<C> {
    fn index_mut(&mut self, StateId(id): StateId) -> &mut Self::Output {
        self.states.index_mut(id)
    }
}

impl Iterator for StateRange {
    type Item = StateId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(StateId)
    }
}
