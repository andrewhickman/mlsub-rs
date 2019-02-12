pub mod flow;
pub mod state;

pub(crate) mod build;

mod reduce;

pub use self::state::{StateId, StateSet, State, StateRange};

pub(crate) use self::flow::FlowSet;

use crate::{Constructor, ConstructorSet, Polarity};

#[derive(Debug)]
pub struct Automaton<C: Constructor> {
    states: Vec<State<C>>,
}

impl<C: Constructor> Automaton<C> {
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

impl<C: Constructor> Default for Automaton<C> {
    fn default() -> Self {
        Automaton::new()
    }
}
