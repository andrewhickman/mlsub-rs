#[cfg(test)]
mod reference;
#[cfg(test)]
mod tests;

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
    pub fn biunify_all<I>(&mut self, constraints: I) -> Result<C>
    where
        I: IntoIterator<Item = (StateId, StateId)>,
    {
        constraints
            .into_iter()
            .try_for_each(|(qp, qn)| self.biunify(qp, qn))
    }

    /// Solves a constraint t⁺ ≤ t⁻ where t⁺ and t⁻ are represented by the states `qp` and `qn`.
    pub fn biunify(&mut self, qp: StateId, qn: StateId) -> Result<C> {
        #[cfg(debug_assertions)]
        debug_assert_eq!(self[qp].pol, Polarity::Pos);
        #[cfg(debug_assertions)]
        debug_assert_eq!(self[qn].pol, Polarity::Neg);

        if self.biunify_cache.insert((qp, qn)) {
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
                for (label, l, r) in
                    merge_join_by(cp.params(), cn.params(), |l, r| Ord::cmp(&l.0, &r.0))
                        .flat_map(|eob| match eob {
                            EitherOrBoth::Both(lc, rc) => Some((lc.0, lc.1, rc.1)),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                {
                    let (ps, ns) = label.polarity().flip(l, r);
                    for (jp, jn) in product(ps, ns) {
                        if let Err(err) = self.biunify(jp, jn) {
                            return Err(err.with(label, cp, cn));
                        }
                    }
                }
            }
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

    fn with(mut self, label: C::Label, cp: C, cn: C) -> Self {
        self.stack.push((label, cp, cn));
        self
    }
}
