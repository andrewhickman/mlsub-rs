#[cfg(test)]
mod reference;
#[cfg(test)]
mod tests;

use std::convert::Infallible;
use std::fmt::{self, Debug};
use std::iter::once;

use crate::auto::{Automaton, StateId};
use crate::{Constructor, Label, Polarity};

pub type Result<C> = std::result::Result<(), Error<C>>;

#[derive(Debug)]
pub struct Error<C: Constructor> {
    pub stack: Vec<(C::Label, C, C)>,
    pub constraint: (C, C),
}

pub(crate) enum CacheEntry<C: Constructor> {
    Root,
    RequiredBy {
        label: C::Label,
        pos: (StateId, C),
        neg: (StateId, C),
    },
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
        stack.extend(constraints.into_iter().filter(|&constraint| {
            self.biunify_cache
                .insert(constraint, CacheEntry::Root)
                .is_none()
        }));
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
        debug_assert!(self.biunify_cache.contains_key(&(qp, qn)));

        for (cp, cn) in product(self[qp].cons.iter(), self[qn].cons.iter()) {
            if !(cp <= cn) {
                return Err(self.make_error((qp, cp.clone()), (qn, cn.clone())));
            }
        }
        for to in self[qn].flow.iter() {
            self.merge(Polarity::Pos, to, qp);
        }
        for from in self[qp].flow.iter() {
            self.merge(Polarity::Neg, from, qn);
        }

        let states = &self.states;
        let biunify_cache = &mut self.biunify_cache;
        let cps = &states[qp.as_u32() as usize].cons;
        let cns = &states[qn.as_u32() as usize].cons;
        for (cp, cn) in cps.intersection(cns) {
            cp.visit_params_intersection::<_, Infallible>(&cn, |label, l, r| {
                let (ps, ns) = label.polarity().flip(l, r);
                stack.extend(product(ps, ns).filter(|&constraint| {
                    biunify_cache
                        .insert(
                            constraint,
                            CacheEntry::RequiredBy {
                                label: label.clone(),
                                pos: (qp, cp.clone()),
                                neg: (qn, cn.clone()),
                            },
                        )
                        .is_none()
                }));
                Ok(())
            })
            .unwrap();
        }
        Ok(())
    }

    fn make_error(&self, pos: (StateId, C), neg: (StateId, C)) -> Error<C> {
        let mut stack = Vec::new();

        let mut key = (pos.0, neg.0);
        while let CacheEntry::RequiredBy { label, pos, neg } = &self.biunify_cache[&key] {
            stack.push((label.clone(), pos.1.clone(), neg.1.clone()));
            key = (pos.0, neg.0);
        }

        Error {
            stack,
            constraint: (pos.1, neg.1),
        }
    }
}

fn product<I, J>(lhs: I, rhs: J) -> impl Iterator<Item = (I::Item, J::Item)>
where
    I: IntoIterator,
    I::Item: Clone + Copy,
    J: IntoIterator,
    J: Clone,
{
    lhs.into_iter()
        .flat_map(move |l| rhs.clone().into_iter().map(move |r| (l.clone(), r)))
}

impl<C> Debug for CacheEntry<C>
where
    C: Constructor + Debug,
    C::Label: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CacheEntry::Root => f.debug_struct("Root").finish(),
            CacheEntry::RequiredBy { label, pos, neg } => f
                .debug_struct("RequiredBy")
                .field("label", label)
                .field("pos", pos)
                .field("neg", neg)
                .finish(),
        }
    }
}
