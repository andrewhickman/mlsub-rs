pub mod flow;
pub mod state;

pub(crate) mod build;

mod reduce;

pub use self::state::{State, StateId, StateRange, StateSet};

pub(crate) use self::flow::FlowSet;

use std::collections::HashSet;
use std::hash::BuildHasherDefault;

use seahash::SeaHasher;

use crate::{Constructor, ConstructorSet, Polarity};

#[derive(Debug)]
pub struct Automaton<C: Constructor> {
    states: Vec<State<C>>,
    pub(crate) biunify_cache: HashSet<(StateId, StateId), BuildHasherDefault<SeaHasher>>,
}

impl<C: Constructor> Automaton<C> {
    pub fn new() -> Self {
        Automaton {
            states: Vec::new(),
            biunify_cache: HashSet::default(),
        }
    }

    pub fn clone_states<I>(&mut self, states: I) -> StateRange
    where
        I: IntoIterator<Item = (StateId, Polarity)>,
    {
        let mut reduced = Automaton::new();
        let range = reduced.reduce(&self, states);

        let offset = self.add_from(&reduced);

        #[cfg(debug_assertions)]
        debug_assert!(self.check_flow());

        range.shift(offset)
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
