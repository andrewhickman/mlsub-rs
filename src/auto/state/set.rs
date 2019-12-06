use std::iter::Copied;
use std::slice;

use small_ord_set::SmallOrdSet;

use crate::auto::state::StateId;

/// A non-empty set of states, optimized for the common case where only one state is in the set.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StateSet {
    set: SmallOrdSet<[StateId; 1]>,
}

impl StateSet {
    pub fn new(id: StateId) -> Self {
        StateSet {
            set: SmallOrdSet::from_buf([id]),
        }
    }

    pub fn insert(&mut self, id: StateId) -> bool {
        self.set.insert(id)
    }

    pub fn union(&mut self, other: &Self) {
        self.set.extend(other)
    }

    pub fn iter(&self) -> Copied<slice::Iter<StateId>> {
        self.set.iter().copied()
    }

    pub(crate) fn shift(self, offset: u32) -> Self {
        StateSet {
            set: self.set.into_iter().map(|id| id.shift(offset)).collect(),
        }
    }

    pub(crate) fn unwrap_reduced(&self) -> StateId {
        debug_assert_eq!(self.set.len(), 1);
        self.set[0]
    }
}

impl<'a> IntoIterator for &'a StateSet {
    type IntoIter = Copied<slice::Iter<'a, StateId>>;
    type Item = StateId;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
