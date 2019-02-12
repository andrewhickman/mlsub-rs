use std::hash::BuildHasherDefault;
use std::iter::{once, Once};

use im::hashset::{ConsumingIter, HashSet};
use seahash::SeaHasher;

use crate::auto::state::StateId;

/// A non-empty set of states, optimized for the common case where only one state is in the set.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StateSet(StateSetData);

#[derive(Debug, Clone, Eq, PartialEq)]
enum StateSetData {
    Singleton(StateId),
    Set(HashSet<StateId, BuildHasherDefault<SeaHasher>>),
}

pub enum StateSetIter {
    Singleton(Once<StateId>),
    Set(ConsumingIter<StateId>),
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