pub mod flow;

mod build;
mod reduce;
mod state;

pub use self::state::{StateId, StateRange, StateSet, StateSetIter};

#[cfg(test)]
pub(crate) use self::build::{Build, Builder};
pub(crate) use self::flow::FlowSet;
pub(crate) use self::state::State;

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
        self.states.iter().all(|st| st.cons.is_reduced())
    }

    pub(crate) fn merge(&mut self, pol: Polarity, target_id: StateId, source_id: StateId) {
        if target_id != source_id {
            let (target, source) = self.index_mut2(target_id, source_id);

            #[cfg(debug_assertions)]
            debug_assert_eq!(target.pol, pol);
            #[cfg(debug_assertions)]
            debug_assert_eq!(source.pol, pol);

            target.cons.merge(&source.cons, pol);
            self.merge_flow(pol, target_id, source_id);
        }
    }
}

impl<T: TypeSystem> Default for Automaton<T> {
    fn default() -> Self {
        Automaton::new()
    }
}
