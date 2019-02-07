pub mod flow;

mod build;
mod reduce;
mod state;
mod trans;

pub use self::build::{Build, Builder};
pub use self::trans::Symbol;

pub(crate) use self::flow::FlowSet;
pub(crate) use self::state::{State, StateId, StateRange};
pub(crate) use self::trans::{Transition, TransitionSet};

use crate::cons::ConstructorSet;
use crate::Polarity;
use crate::TypeSystem;

#[derive(Debug)]
pub struct Automaton<T: TypeSystem> {
    states: Vec<State<T>>,
}

impl<T: TypeSystem> Automaton<T> {
    pub fn new() -> Self {
        Automaton { states: Vec::new() }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn is_reduced(&self) -> bool {
        self.states.iter().all(|st| st.trans.is_reduced())
    }

    pub(crate) fn merge(&mut self, pol: Polarity, target: StateId, source: StateId) {
        match pol {
            Polarity::Pos => self.merge_pos(target, source),
            Polarity::Neg => self.merge_neg(target, source),
        }
    }

    pub(crate) fn merge_pos(&mut self, target_id: StateId, source_id: StateId) {
        if target_id != source_id {
            let (target, source) = self.index_mut2(target_id, source_id);

            #[cfg(debug_assertions)]
            debug_assert_eq!(target.pol, Polarity::Pos);
            #[cfg(debug_assertions)]
            debug_assert_eq!(source.pol, Polarity::Pos);

            target.cons.join(&source.cons);
            target.trans.union(&source.trans);
            self.merge_flow_pos(target_id, source_id);
        }
    }

    pub(crate) fn merge_neg(&mut self, target_id: StateId, source_id: StateId) {
        if target_id != source_id {
            let (target, source) = self.index_mut2(target_id, source_id);

            #[cfg(debug_assertions)]
            debug_assert_eq!(target.pol, Polarity::Neg);
            #[cfg(debug_assertions)]
            debug_assert_eq!(source.pol, Polarity::Neg);

            target.cons.meet(&source.cons);
            target.trans.union(&source.trans);
            self.merge_flow_neg(target_id, source_id);
        }
    }
}

impl<T: TypeSystem> Default for Automaton<T> {
    fn default() -> Self {
        Automaton::new()
    }
}
