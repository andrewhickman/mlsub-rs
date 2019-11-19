#[cfg(test)]
mod reference;
#[cfg(test)]
mod tests;

use std::iter::once;

use crate::auto::{Automaton, StateId};
use crate::{Constructor, Label, Polarity};
use itertools::{merge_join_by, EitherOrBoth};

pub type Result<C> = std::result::Result<(), Error<C>>;

#[derive(Debug)]
pub struct Error<C: Constructor> {
    pub stack: Vec<(C::Label, C, C)>,
    pub constraint: (C, C),
}

impl<C: Constructor> Automaton<C> {
    /// Solves a set of constraints t⁺ ≤ t⁻ where t⁺ and t⁻ are represented by the states `qp` and `qn`.
    pub fn biunify(&mut self, qp: StateId, qn: StateId) -> Result<C> {
        self.biunify_all(once((qp, qn)))
    }

    /// Solves a set of constraints t⁺ ≤ t⁻ where t⁺ and t⁻ are represented by the states `qp` and `qn`.
    pub fn biunify_all<I>(&mut self, constraints: I) -> Result<C>
    where
        I: IntoIterator<Item = (StateId, StateId)>,
    {
        let mut stack = Vec::with_capacity(20);
        stack.extend(
            constraints
                .into_iter()
                .filter(|&constraint| self.biunify_cache.insert(constraint)),
        );
        while let Some(constraint) = stack.pop() {
            self.biunify_impl(&mut stack, constraint)?;
        }
        Ok(())
    }

    fn biunify_impl(
        &mut self,
        stack: &mut Vec<(StateId, StateId)>,
        (qp, qn): (StateId, StateId),
    ) -> Result<C> {
        #[cfg(debug_assertions)]
        debug_assert_eq!(self[qp].pol, Polarity::Pos);
        #[cfg(debug_assertions)]
        debug_assert_eq!(self[qn].pol, Polarity::Neg);
        debug_assert!(self.biunify_cache.contains(&(qp, qn)));

        for (cp, cn) in product(&self[qp].cons, &self[qn].cons) {
            if !(cp <= cn) {
                return Err(Error::new(cp.clone(), cn.clone()));
            }
        }
        for to in self[qn].flow.iter() {
            self.merge(Polarity::Pos, to, qp);
        }
        for from in self[qp].flow.iter() {
            self.merge(Polarity::Neg, from, qn);
        }
        let cps = self[qp].cons.clone();
        let cns = self[qn].cons.clone();
        for (cp, cn) in cps.intersection(cns) {
            stack.extend(
                merge_join_by(cp.params(), cn.params(), |l, r| Ord::cmp(&l.0, &r.0))
                    .flat_map(|eob| match eob {
                        EitherOrBoth::Both(lc, rc) => Some((lc.0, lc.1, rc.1)),
                        _ => None,
                    })
                    .flat_map(|(label, l, r)| {
                        let (ps, ns) = label.polarity().flip(l, r);
                        product(ps, ns)
                    })
                    .filter(|&constraint| self.biunify_cache.insert(constraint)),
            )
        }
        Ok(())
    }
}

fn product<I, J>(lhs: I, rhs: J) -> impl Iterator<Item = (I::Item, J::Item)>
where
    I: IntoIterator,
    I::Item: Clone,
    J: IntoIterator,
    J: Clone,
{
    lhs.into_iter()
        .flat_map(move |l| rhs.clone().into_iter().map(move |r| (l.clone(), r)))
}

impl<C: Constructor> Error<C> {
    fn new(cp: C, cn: C) -> Self {
        Error {
            stack: vec![],
            constraint: (cp, cn),
        }
    }

    // TODO: use
    fn with(mut self, label: C::Label, cp: C, cn: C) -> Self {
        self.stack.push((label, cp, cn));
        self
    }
}
