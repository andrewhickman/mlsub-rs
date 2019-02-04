use std::iter::Flatten;
use std::option;

use im::ordset::{ConsumingIter, OrdSet};

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
    set: Option<OrdSet<Transition<S>>>,
}

impl<S: Symbol> Transition<S> {
    pub(crate) fn symbol(&self) -> S {
        self.symbol.clone()
    }

    pub(crate) fn id(&self) -> StateId {
        self.id
    }
}

impl<S: Symbol> TransitionSet<S> {
    pub(crate) fn add(&mut self, symbol: S, id: StateId) {
        self.set().insert(Transition { symbol, id });
    }

    pub(crate) fn union(&mut self, other: &Self) {
        self.set().extend(other.clone())
    }

    #[cfg(debug_assertions)]
    pub(crate) fn is_reduced(&self) -> bool {
        use itertools::Itertools;

        match &self.set {
            Some(set) => set
                .iter()
                .group_by(|tr| &tr.symbol)
                .into_iter()
                .all(|(_, group)| group.count() == 1),
            None => true,
        }
    }

    fn set(&mut self) -> &mut OrdSet<Transition<S>> {
        self.set.get_or_insert_with(Default::default)
    }
}

impl<S: Symbol> Extend<(S, StateId)> for TransitionSet<S> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = (S, StateId)>,
    {
        self.set().extend(
            iter.into_iter()
                .map(|(symbol, id)| Transition { symbol, id }),
        )
    }
}

impl<S: Symbol> Default for TransitionSet<S> {
    fn default() -> Self {
        TransitionSet { set: None }
    }
}

impl<'a, S: Symbol> IntoIterator for TransitionSet<S> {
    type Item = Transition<S>;
    type IntoIter = Flatten<option::IntoIter<ConsumingIter<Transition<S>>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.set.map(OrdSet::into_iter).into_iter().flatten()
    }
}
