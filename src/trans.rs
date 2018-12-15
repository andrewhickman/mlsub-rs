use im::{ordset, OrdSet};

use crate::auto::StateId;
use crate::Polarity;

pub trait Symbol: Clone + Ord {
    fn polarity(&self) -> Polarity;
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub(crate) struct Transition<S> {
    symbol: S,
    id: StateId,
}

pub(crate) struct TransitionSet<S> {
    set: OrdSet<Transition<S>>,
}

impl<S: Symbol> Transition<S> {
    pub(crate) fn new(symbol: S, id: StateId) -> Self {
        Transition { symbol, id }
    }

    pub(crate) fn symbol(&self) -> S {
        self.symbol.clone()
    }

    pub(crate) fn id(&self) -> StateId {
        self.id
    }
}

impl<S: Symbol> TransitionSet<S> {
    pub(crate) fn add(&mut self, tr: Transition<S>) {
        self.set.insert(tr);
    }

    pub(crate) fn union(&mut self, other: &Self) {
        self.set = self.set.clone().union(other.set.clone());
    }

    pub(crate) fn iter(&self) -> ordset::ConsumingIter<Transition<S>> {
        self.set.clone().into_iter()
    }
}

impl<S: Symbol> Default for TransitionSet<S> {
    fn default() -> Self {
        TransitionSet {
            set: OrdSet::default(),
        }
    }
}
