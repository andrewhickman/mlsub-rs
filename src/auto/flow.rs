use std::hash::BuildHasherDefault;
use std::iter::FromIterator;

use im::{hashset, HashSet};
use seahash::SeaHasher;
use once_cell::sync::Lazy;

use crate::auto::{Automaton, StateId};
use crate::{Constructor, Polarity};

#[derive(Copy, Clone, Debug)]
pub struct Pair {
    pub neg: StateId,
    pub pos: StateId,
}

#[derive(Debug, Clone)]
pub struct FlowSet {
    set: HashSet<StateId, BuildHasherDefault<SeaHasher>>,
}

impl Pair {
    pub(crate) fn from_pol(pol: Polarity, a: StateId, b: StateId) -> Self {
        let (pos, neg) = pol.flip(a, b);
        Pair { pos, neg }
    }

    pub fn get(&self, pol: Polarity) -> StateId {
        match pol {
            Polarity::Pos => self.pos,
            Polarity::Neg => self.neg,
        }
    }
}

impl FlowSet {
    pub fn iter(&self) -> hashset::ConsumingIter<StateId> {
        self.set.clone().into_iter()
    }

    pub(in crate::auto) fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = StateId>,
    {
        FlowSet {
            set: HashSet::from_iter(iter),
        }
    }

    pub(crate) fn shift(self, offset: u32) -> Self {
        FlowSet::from_iter(self.set.into_iter().map(|id| id.shift(offset)))
    }

    pub(in crate::auto) fn union(&mut self, other: &Self) {
        self.set.extend(other.iter());
    }
}

impl Default for FlowSet {
    fn default() -> Self {
        static EMPTY: Lazy<FlowSet> = Lazy::new(|| FlowSet {
            set: HashSet::default()
        });

        EMPTY.clone()
    }
}

impl<C: Constructor> Automaton<C> {
    pub(crate) fn add_flow(&mut self, pair: Pair) {
        #[cfg(debug_assertions)]
        debug_assert_eq!(self[pair.pos].pol, Polarity::Pos);
        #[cfg(debug_assertions)]
        debug_assert_eq!(self[pair.neg].pol, Polarity::Neg);

        let had_p = self[pair.pos].flow.set.insert(pair.neg).is_some();
        let had_n = self[pair.neg].flow.set.insert(pair.pos).is_some();
        debug_assert_eq!(had_p, had_n);
    }

    pub(crate) fn remove_flow(&mut self, pair: Pair) {
        #[cfg(debug_assertions)]
        debug_assert_eq!(self[pair.pos].pol, Polarity::Pos);
        #[cfg(debug_assertions)]
        debug_assert_eq!(self[pair.neg].pol, Polarity::Neg);

        let had_p = self[pair.pos].flow.set.remove(&pair.neg).is_some();
        let had_n = self[pair.neg].flow.set.remove(&pair.pos).is_some();
        debug_assert_eq!(had_p, had_n);
    }

    pub(crate) fn has_flow(&self, pair: Pair) -> bool {
        #[cfg(debug_assertions)]
        debug_assert_eq!(self[pair.pos].pol, Polarity::Pos);
        #[cfg(debug_assertions)]
        debug_assert_eq!(self[pair.neg].pol, Polarity::Neg);

        self[pair.neg].flow.set.contains(&pair.pos)
    }

    pub(crate) fn merge_flow(&mut self, pol: Polarity, a: StateId, source: StateId) {
        #[cfg(debug_assertions)]
        debug_assert_eq!(self[a].pol, pol);
        #[cfg(debug_assertions)]
        debug_assert_eq!(self[source].pol, pol);

        for b in self[source].flow.iter() {
            self.add_flow(Pair::from_pol(pol, a, b));
        }
    }

    #[cfg(debug_assertions)]
    pub(in crate::auto) fn check_flow(&self) -> bool {
        self.enumerate().all(|(from, st)| {
            st.flow
                .iter()
                .all(|to| self[to].pol != st.pol && self[to].flow.set.contains(&from))
        })
    }
}
