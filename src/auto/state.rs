use std::hash::BuildHasherDefault;
use std::iter::{once, Once};
use std::ops::Range;

use im::hashset::{ConsumingIter, HashSet};
use seahash::SeaHasher;

use crate::auto::{Automaton, ConstructorSet, FlowSet};
use crate::{Polarity, TypeSystem};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StateId(usize);

#[derive(Debug, Clone)]
pub struct StateRange(Range<usize>);

#[derive(Debug)]
pub(crate) struct State<T: TypeSystem> {
    #[cfg(debug_assertions)]
    pub(crate) pol: Polarity,
    pub(crate) cons: ConstructorSet<T::Constructor>,
    pub(crate) flow: FlowSet,
}

/// A non-empty set of states, optimized for the common case where only one state is in the set.
// TODO priv enum
#[derive(Debug, Clone)]
pub enum StateSet {
    Singleton(StateId),
    Set(HashSet<StateId, BuildHasherDefault<SeaHasher>>),
}

pub enum StateSetIter {
    Singleton(Once<StateId>),
    Set(ConsumingIter<StateId>),
}

impl<T: TypeSystem> State<T> {
    pub(crate) fn new(_pol: Polarity) -> Self {
        State {
            #[cfg(debug_assertions)]
            pol: _pol,
            cons: ConstructorSet::default(),
            flow: FlowSet::default(),
        }
    }
}

impl<T: TypeSystem> Clone for State<T> {
    fn clone(&self) -> Self {
        State {
            #[cfg(debug_assertions)]
            pol: self.pol,
            cons: self.cons.clone(),
            flow: self.flow.clone(),
        }
    }
}

impl<T: TypeSystem> Automaton<T> {
    pub(crate) fn next(&mut self) -> StateId {
        StateId(self.states.len())
    }

    pub(crate) fn add(&mut self, state: State<T>) -> StateId {
        let id = self.next();
        self.states.push(state);
        id
    }

    pub(crate) fn index(&self, StateId(id): StateId) -> &State<T> {
        &self.states[id]
    }

    pub(crate) fn index_mut(&mut self, StateId(id): StateId) -> &mut State<T> {
        &mut self.states[id]
    }

    pub(crate) fn index_mut2(
        &mut self,
        StateId(i): StateId,
        StateId(j): StateId,
    ) -> (&mut State<T>, &mut State<T>) {
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

    pub(crate) fn enumerate(&self) -> impl Iterator<Item = (StateId, &State<T>)> {
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
    pub fn insert(&mut self, id: StateId) -> bool {
        self.to_set().insert(id).is_some()
    }

    pub fn union(&mut self, other: &Self) {
        match other {
            StateSet::Singleton(id) => {
                self.insert(*id);
            }
            StateSet::Set(set) => self.to_set().extend(set.iter().cloned()),
        }
    }

    pub fn iter(&self) -> StateSetIter {
        match self {
            StateSet::Singleton(id) => StateSetIter::Singleton(once(*id)),
            StateSet::Set(set) => StateSetIter::Set(set.clone().into_iter()),
        }
    }

    pub fn to_set(&mut self) -> &mut HashSet<StateId, BuildHasherDefault<SeaHasher>> {
        if let StateSet::Singleton(id) = *self {
            *self = StateSet::Set(HashSet::default().update(id));
        }
        match self {
            StateSet::Set(set) => set,
            StateSet::Singleton(_) => unreachable!(),
        }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn is_reduced(&self) -> bool {
        match self {
            StateSet::Singleton(_) => true,
            StateSet::Set(_) => false,
        }
    }
}

impl PartialEq for StateSet {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (StateSet::Set(lhs), StateSet::Set(rhs)) => lhs == rhs,
            _ => self.iter().eq(other.iter()),
        }
    }
}

impl Eq for StateSet {}

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
