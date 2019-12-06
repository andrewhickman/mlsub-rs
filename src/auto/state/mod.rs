mod set;

pub use self::set::StateSet;

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
pub struct StateId(u32);

#[derive(Debug, Clone)]
pub struct StateRange(Range<u32>);

impl StateId {
    pub fn as_u32(self) -> u32 {
        self.0
    }

    pub fn shift(self, offset: u32) -> Self {
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

    pub fn flow(&self) -> &FlowSet {
        &self.flow
    }

    fn shift(self, offset: u32) -> Self {
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
        StateId(self.states.len() as u32)
    }

    pub(crate) fn add(&mut self, state: State<C>) -> StateId {
        let id = self.next();
        self.states.push(state);
        id
    }

    pub fn add_from(&mut self, other: &Self) -> u32 {
        let offset = self.states.len() as u32;
        self.states.extend(
            other
                .states
                .iter()
                .cloned()
                .map(|state| state.shift(offset)),
        );
        offset
    }

    pub(crate) fn index_mut2(
        &mut self,
        StateId(i): StateId,
        StateId(j): StateId,
    ) -> (&mut State<C>, &mut State<C>) {
        debug_assert_ne!(i, j);
        if i < j {
            let (l, r) = self.states.split_at_mut(j as usize);
            (&mut l[i as usize], &mut r[0])
        } else {
            let (l, r) = self.states.split_at_mut(i as usize);
            (&mut r[0], &mut l[j as usize])
        }
    }

    pub(crate) fn range_from(&mut self, StateId(start): StateId) -> StateRange {
        StateRange(start..(self.states.len() as u32))
    }

    pub(crate) fn enumerate(&self) -> impl Iterator<Item = (StateId, &State<C>)> {
        self.states
            .iter()
            .enumerate()
            .map(|(id, st)| (StateId(id as u32), st))
    }
}

impl<C: Constructor> Index<StateId> for Automaton<C> {
    type Output = State<C>;

    fn index(&self, StateId(id): StateId) -> &Self::Output {
        self.states.index(id as usize)
    }
}

impl<C: Constructor> IndexMut<StateId> for Automaton<C> {
    fn index_mut(&mut self, StateId(id): StateId) -> &mut Self::Output {
        self.states.index_mut(id as usize)
    }
}

impl StateRange {
    pub fn shift(self, offset: u32) -> Self {
        StateRange((self.0.start + offset)..(self.0.end + offset))
    }
}

impl Iterator for StateRange {
    type Item = StateId;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(StateId)
    }
}
