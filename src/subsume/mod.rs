use std::collections::HashSet;
use std::hash::BuildHasherDefault;

use seahash::SeaHasher;

use crate::auto::{flow, Automaton, StateId};
use crate::TypeSystem;

impl<T: TypeSystem> Automaton<T> {
    pub fn subsume(&mut self, a: StateId, b: StateId) -> Result<(), ()> {
        #[cfg(debug_assertions)]
        debug_assert!(self.is_reduced());

        let mut seen = HashSet::with_capacity_and_hasher(20, Default::default());
        self.subsume_impl(&mut seen, a, b)
    }

    fn subsume_impl(
        &mut self,
        seen: &mut HashSet<(StateId, StateId), BuildHasherDefault<SeaHasher>>,
        a: StateId,
        b: StateId,
    ) -> Result<(), ()> {
        if seen.insert((a, b)) {
            unimplemented!()
        }
        Ok(())
    }

    pub fn admissible(&mut self, pair: flow::Pair) -> bool {
        #[cfg(debug_assertions)]
        debug_assert!(self.is_reduced());

        if self.has_flow(pair) {
            true
        } else {
            self.add_flow(pair);

            unimplemented!();

            self.remove_flow(pair);
            false
        }
    }
}
