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

#[derive(Debug, Clone)]
pub(crate) struct TransitionSet<S: Symbol> {
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
        self.set.extend(other.clone())
    }
}

impl<S: Symbol> Default for TransitionSet<S> {
    fn default() -> Self {
        TransitionSet {
            set: OrdSet::default(),
        }
    }
}

impl<'a, S: Symbol> IntoIterator for TransitionSet<S> {
    type Item = Transition<S>;
    type IntoIter = ordset::ConsumingIter<Transition<S>>;

    fn into_iter(self) -> Self::IntoIter {
        self.set.into_iter()
    }
}
