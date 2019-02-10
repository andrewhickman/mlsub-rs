use std::hash::BuildHasherDefault;
use std::iter::{once, Once};
use std::ops::Range;

use im::hashset::{ConsumingIter, HashSet};
use seahash::SeaHasher;

use crate::auto::{Automaton, ConstructorSet, FlowSet};
use crate::{Constructor, Polarity};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StateId(usize);

#[derive(Debug, Clone)]
pub struct StateRange(Range<usize>);

#[derive(Debug)]
pub(crate) struct State<C: Constructor> {
    #[cfg(debug_assertions)]
    pub(crate) pol: Polarity,
    pub(crate) cons: ConstructorSet<C>,
    pub(crate) flow: FlowSet,
}

/// A non-empty set of states, optimized for the common case where only one state is in the set.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StateSet(StateSetData);

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StateSetData {
    Singleton(StateId),
    Set(HashSet<StateId, BuildHasherDefault<SeaHasher>>),
}

pub enum StateSetIter {
    Singleton(Once<StateId>),
    Set(ConsumingIter<StateId>),
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

impl StateSet {
    pub fn new(id: StateId) -> Self {
        StateSet(StateSetData::Singleton(id))
    }

    pub fn insert(&mut self, id: StateId) -> bool {
        self.to_set().insert(id).is_some()
    }

    pub fn union(&mut self, other: &Self) {
        match &other.0 {
            StateSetData::Singleton(id) => {
                self.insert(*id);
            }
            StateSetData::Set(set) => {
                debug_assert!(set.len() > 1);
                self.to_set().extend(set.iter().cloned())
            },
        }
    }

    pub fn iter(&self) -> StateSetIter {
        match &self.0 {
            StateSetData::Singleton(id) => StateSetIter::Singleton(once(*id)),
            StateSetData::Set(set) => StateSetIter::Set(set.clone().into_iter()),
        }
    }

    fn to_set(&mut self) -> &mut HashSet<StateId, BuildHasherDefault<SeaHasher>> {
        if let StateSetData::Singleton(id) = self.0 {
            self.0 = StateSetData::Set(HashSet::default().update(id));
        }
        match &mut self.0 {
            StateSetData::Set(set) => set,
            StateSetData::Singleton(_) => unreachable!(),
        }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn is_reduced(&self) -> bool {
        match self.0 {
            StateSetData::Singleton(_) => true,
            StateSetData::Set(_) => false,
        }
    }

    pub(crate) fn unwrap_reduced(&self) -> StateId {
        match self.0 {
            StateSetData::Singleton(id) => id,
            StateSetData::Set(_) => panic!("not reduced"),
        }
    }
}

impl IntoIterator for StateSet {
    type IntoIter = StateSetIter;
    type Item = StateId;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl Iterator for StateSetIter {
    type Item = StateId;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            StateSetIter::Singleton(iter) => iter.next(),
            StateSetIter::Set(iter) => iter.next(),
        }
    }
}
