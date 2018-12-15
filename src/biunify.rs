use std::collections::HashSet;
use std::hash::BuildHasherDefault;
use std::iter::FromIterator;

use seahash::SeaHasher;
use itertools::{iproduct, merge_join_by, EitherOrBoth, Itertools};

use crate::auto::{Automaton, StateId};
use crate::trans::{Symbol, Transition};
use crate::{Polarity, TypeSystem};

impl<T: TypeSystem> Automaton<T> {
    /// Solves a constraint t⁺ ≤ t⁻ where t⁺ and t⁻ are represented by the states `qp` and `qn`.
    pub fn biunify(&mut self, qp: StateId, qn: StateId) -> Result<(), ()> {
        let mut seen = HashSet::with_capacity_and_hasher(20, Default::default());
        self.biunify_impl(&mut seen, qp, qn)
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
            if !iproduct!(&self.index(qp).cons, &self.index(qn).cons).all(|(l, r)| l <= r) {
                return Err(());
            }
            for to in self.index(qn).flow.iter() {
                self.merge_pos(to, qp);
            }
            for from in self.index(qp).flow.iter() {
                self.merge_neg(from, qn);
            }
            visit_common_groups_by(
                self.index(qp).trans.iter(),
                self.index(qn).trans.iter(),
                Transition::symbol,
                |key, g0, g1| {
                    let (gp, gn) = match key.polarity() {
                        Polarity::Pos => (g0, g1),
                        Polarity::Neg => (g1, g0),
                    };
                    for (jp, jn) in iproduct!(gp, gn) {
                        self.biunify_impl(seen, jp.id(), jn.id())?;
                    }
                    Ok(())
                },
            )?;
        }
        Ok(())
    }
}

/// Take two iterators sorted by a key, group them and yield common groups and then call the given
/// function on them.
fn visit_common_groups_by<I, K, C, E, F>(lhs: I, rhs: I, key: C, mut f: F) -> Result<(), E>
where
    I: Iterator,
    K: Ord + Eq,
    C: Fn(&I::Item) -> K,
    F: for<'a> FnMut(K, &[I::Item], &[I::Item]) -> Result<(), E>,
{
    let lhs = lhs.group_by(&key);
    let rhs = rhs.group_by(&key);
    for pair in merge_join_by(&lhs, &rhs, |l, r| Ord::cmp(&l.0, &r.0)) {
        if let EitherOrBoth::Both((key, l), (_, r)) = pair {
            f(key, &Vec::from_iter(l), &Vec::from_iter(r))?;
        }
    }
    Ok(())
}
