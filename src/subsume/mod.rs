use std::collections::HashSet;
use std::hash::BuildHasherDefault;

use itertools::{merge_join_by, EitherOrBoth};
use seahash::SeaHasher;

use crate::auto::{flow, Automaton, StateId};
use crate::{Constructor, TypeSystem};

impl<T: TypeSystem> Automaton<T> {
    pub fn subsume(&self, a: StateId, b: StateId) -> Result<(), ()> {
        #[cfg(debug_assertions)]
        debug_assert!(self.is_reduced());

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
        debug_assert_eq!(self.index(a).pol, self.index(b).pol);

        if seen.insert((a, b)) {
            for lcon in &self.index(a).cons {
                let rcon = match self.index(b).cons.get(lcon.component()) {
                    Some(rcon) if lcon <= rcon => rcon,
                    _ => return Err(()),
                };

                for pair in merge_join_by(lcon.params(), rcon.params(), |l, r| Ord::cmp(&l.0, &r.0))
                {
                    if let EitherOrBoth::Both((_, l), (_, r)) = pair {
                        self.subsume_impl(seen, l.unwrap_reduced(), r.unwrap_reduced())?;
                    }
                }
            }
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
