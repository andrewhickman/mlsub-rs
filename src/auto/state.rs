use std::ops::Range;

use crate::auto::{Automaton, ConstructorSet, FlowSet, TransitionSet};
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
    pub(crate) trans: TransitionSet<T::Symbol>,
    pub(crate) flow: FlowSet,
}

impl<T: TypeSystem> State<T> {
    pub(crate) fn new(_pol: Polarity) -> Self {
        State {
            #[cfg(debug_assertions)]
            pol: _pol,
            cons: ConstructorSet::default(),
            trans: TransitionSet::default(),
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
            trans: self.trans.clone(),
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
