pub mod flow;
pub mod state;

pub(crate) mod build;

mod reduce;

pub use self::build::Build;
pub use self::state::{State, StateId, StateRange, StateSet};

pub(crate) use self::flow::FlowSet;

use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::hash::BuildHasherDefault;

use seahash::SeaHasher;

use crate::{biunify, Constructor, ConstructorSet, Polarity};

pub struct Automaton<C: Constructor> {
    pub(crate) states: Vec<State<C>>,
    pub(crate) biunify_cache:
        HashMap<(StateId, StateId), biunify::CacheEntry<C>, BuildHasherDefault<SeaHasher>>,
}

impl<C: Constructor> Automaton<C> {
    pub fn new() -> Self {
        Automaton {
            states: Vec::new(),
            biunify_cache: HashMap::default(),
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

impl<C> fmt::Debug for Automaton<C>
where
    C: Constructor + Debug,
    C::Label: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Automaton")
            .field("states", &self.states)
            .field("biunify_cache", &self.biunify_cache)
            .finish()
    }
}
