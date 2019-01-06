mod build;
#[cfg(test)]
mod cmp;
mod flow;
mod reduce;
mod trans;

pub use self::build::{Build, Builder};
pub use self::trans::Symbol;

pub(crate) use self::flow::FlowSet;
pub(crate) use self::trans::{Transition, TransitionSet};

use crate::cons::ConstructorSet;
use crate::Polarity;
use crate::TypeSystem;

pub type StateId = usize;

#[derive(Debug)]
pub(crate) struct State<T: TypeSystem> {
    #[cfg(debug_assertions)]
    pub(crate) pol: Polarity,
    pub(crate) cons: ConstructorSet<T::Constructor>,
    pub(crate) trans: TransitionSet<T::Symbol>,
    pub(crate) flow: FlowSet,
}

#[derive(Debug)]
pub struct Automaton<T: TypeSystem> {
    states: Vec<State<T>>,
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

impl<T: TypeSystem> Automaton<T> {
    pub fn new() -> Self {
        Automaton { states: Vec::new() }
    }

    pub(crate) fn add(&mut self, state: State<T>) -> StateId {
        let id = self.states.len();
        self.states.push(state);
        id
    }

    pub(crate) fn index(&self, id: StateId) -> &State<T> {
        &self.states[id]
    }

    #[cfg(debug_assertions)]
    pub(crate) fn is_reduced(&self) -> bool {
        self.states.iter().all(|st| st.trans.is_reduced())
    }

    pub(crate) fn index_mut(&mut self, id: StateId) -> &mut State<T> {
        &mut self.states[id]
    }

    pub(crate) fn merge(&mut self, pol: Polarity, target: StateId, source: StateId) {
        match pol {
            Polarity::Pos => self.merge_pos(target, source),
            Polarity::Neg => self.merge_neg(target, source),
        }
    }

    pub(crate) fn merge_pos(&mut self, target_id: StateId, source_id: StateId) {
        let (target, source) = index2(&mut self.states, target_id, source_id);

        #[cfg(debug_assertions)]
        debug_assert_eq!(target.pol, Polarity::Pos);
        #[cfg(debug_assertions)]
        debug_assert_eq!(source.pol, Polarity::Pos);

        target.cons.join(&source.cons);
        target.trans.union(&source.trans);
        self.merge_flow_pos(target_id, source_id);
    }

    pub(crate) fn merge_neg(&mut self, target_id: StateId, source_id: StateId) {
        let (target, source) = index2(&mut self.states, target_id, source_id);

        #[cfg(debug_assertions)]
        debug_assert_eq!(target.pol, Polarity::Neg);
        #[cfg(debug_assertions)]
        debug_assert_eq!(source.pol, Polarity::Neg);

        target.cons.meet(&source.cons);
        target.trans.union(&source.trans);
        self.merge_flow_neg(target_id, source_id);
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

fn index2<T>(slice: &mut [T], i: usize, j: usize) -> (&mut T, &mut T) {
    if i < j {
        let (l, r) = slice.split_at_mut(j);
        (&mut l[i], &mut r[0])
    } else {
        let (l, r) = slice.split_at_mut(i);
        (&mut r[0], &mut l[j])
    }
}
