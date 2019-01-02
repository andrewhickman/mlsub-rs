use std::collections::HashSet;
use std::hash::BuildHasherDefault;

use seahash::SeaHasher;

use crate::auto::{Automaton, StateId, Symbol, TransitionSet};
use crate::{Polarity, TypeSystem};

impl<T: TypeSystem> Automaton<T> {
    /// Solves a constraint t⁺ ≤ t⁻ where t⁺ and t⁻ are represented by the states `qp` and `qn`.
    pub fn biunify(&mut self, qp: StateId, qn: StateId) -> bool {
        let mut seen = HashSet::with_capacity_and_hasher(20, Default::default());
        self.biunify_impl(&mut seen, qp, qn).is_ok()
    }

    fn biunify_impl(
        &mut self,
        seen: &mut HashSet<(StateId, StateId), BuildHasherDefault<SeaHasher>>,
        qp: StateId,
        qn: StateId,
    ) -> Result<(), ()> {
        #[cfg(debug_assertions)]
        debug_assert_eq!(self.index(qp).pol, Polarity::Pos);
        #[cfg(debug_assertions)]
        debug_assert_eq!(self.index(qn).pol, Polarity::Neg);

        if seen.insert((qp, qn)) {
            if !product(&self.index(qp).cons, &self.index(qn).cons).all(|(l, r)| l <= r) {
                return Err(());
            }
            for to in self.index(qn).flow.iter() {
                self.merge_pos(to, qp);
            }
            for from in self.index(qp).flow.iter() {
                self.merge_neg(from, qn);
            }
            let jps = self.index(qp).trans.clone();
            let jns = self.index(qn).trans.clone();
            for (jp, jn) in common_groups(jps, jns) {
                self.biunify_impl(seen, jp, jn)?;
            }
        }
        Ok(())
    }
}

fn common_groups<S>(
    lhs: TransitionSet<S>,
    rhs: TransitionSet<S>,
) -> impl Iterator<Item = (StateId, StateId)>
where
    S: Symbol,
{
    product(lhs, rhs)
        .filter(|(l, r)| l.symbol() == r.symbol())
        .map(|(l, r)| match l.symbol().polarity() {
            Polarity::Pos => (l.id(), r.id()),
            Polarity::Neg => (r.id(), l.id()),
        })
}

fn product<I, J>(lhs: I, rhs: J) -> impl Iterator<Item = (I::Item, J::Item)> 
where 
    I: IntoIterator,
    I::Item: Clone,
    J: IntoIterator,
    J: Clone,
{
    lhs.into_iter().flat_map(move |l| rhs.clone().into_iter().map(move |r| (l.clone(), r)))
}