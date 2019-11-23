use std::collections::HashSet;
use std::hash::BuildHasherDefault;

use seahash::SeaHasher;

use crate::auto::{flow, Automaton, StateId};
use crate::Constructor;

impl<C: Constructor> Automaton<C> {
    pub fn subsume(&self, a: StateId, b: StateId) -> Result<(), ()> {
        let mut seen = HashSet::with_capacity_and_hasher(20, Default::default());
        self.subsume_impl(&mut seen, a, b)
    }

    fn subsume_impl(
        &self,
        seen: &mut HashSet<(StateId, StateId), BuildHasherDefault<SeaHasher>>,
        a: StateId,
        b: StateId,
    ) -> Result<(), ()> {
        #[cfg(debug_assertions)]
        debug_assert_eq!(self[a].pol, self[b].pol);

        if seen.insert((a, b)) {
            for lcon in self[a].cons.iter() {
                let rcon = match self[b].cons.get(lcon.component()) {
                    Some(rcon) if lcon <= rcon => rcon,
                    _ => return Err(()),
                };

                lcon.visit_params_intersection(rcon, |_, l, r| {
                    self.subsume_impl(seen, l.unwrap_reduced(), r.unwrap_reduced())
                })?;
            }
        }
        Ok(())
    }

    pub fn admissible(&mut self, pair: flow::Pair) -> bool {
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
