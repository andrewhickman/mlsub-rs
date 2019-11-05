#[cfg(test)]
mod reference;
#[cfg(test)]
mod tests;

use crate::auto::{Automaton, StateId};
use crate::{Constructor, ConstructorSet, Label, Polarity};

pub type Result<C> = std::result::Result<(), Error<C>>;

#[derive(Debug)]
pub struct Error<C: Constructor> {
    pub stack: Vec<(C::Label, ConstructorSet<C>, ConstructorSet<C>)>,
    pub constraint: (ConstructorSet<C>, ConstructorSet<C>),
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
            if !product(&self[qp].cons, &self[qn].cons).all(|(l, r)| l <= r) {
                return Err(Error::new(self, qp, qn));
            }
            for to in self[qn].flow.iter() {
                self.merge(Polarity::Pos, to, qp);
            }
            for from in self[qp].flow.iter() {
                self.merge(Polarity::Neg, from, qn);
            }
            let jps = self[qp].cons.clone();
            let jns = self[qn].cons.clone();
            for (label, l, r) in jps.intersection(jns) {
                let (ps, ns) = label.polarity().flip(l, r);
                for (jp, jn) in product(ps, ns) {
                    if let Err(err) = self.biunify(jp, jn) {
                        return Err(err.with(self, label, qp, qn));
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
    fn new(auto: &Automaton<C>, qp: StateId, qn: StateId) -> Self {
        Error {
            stack: vec![],
            constraint: (auto[qp].cons.clone(), auto[qn].cons.clone()),
        }
    }

    fn with(mut self, auto: &Automaton<C>, label: C::Label, qp: StateId, qn: StateId) -> Self {
        self.stack
            .push((label, auto[qp].cons.clone(), auto[qn].cons.clone()));
        self
    }
}
