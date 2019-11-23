mod arbitrary;
mod build;

pub use self::arbitrary::{arb_auto_ty, arb_polar_ty};
pub use self::build::Constructed;

use std::cmp::Ordering;
use std::rc::Rc;
use std::vec;

use im::OrdMap;
use itertools::EitherOrBoth;

use crate::auto::StateSet;
use crate::Polarity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Constructor {
    Bool,
    Fun(StateSet, StateSet),
    Record(OrdMap<Rc<str>, StateSet>),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Component {
    Bool,
    Fun,
    Record,
}

impl crate::Constructor for Constructor {
    type Label = Label;
    type Component = Component;

    fn component(&self) -> Self::Component {
        match self {
            Constructor::Bool => Component::Bool,
            Constructor::Fun(..) => Component::Fun,
            Constructor::Record(..) => Component::Record,
        }
    }

    fn join(&mut self, other: &Self, pol: Polarity) {
        match (self, other) {
            (Constructor::Bool, Constructor::Bool) => (),
            (Constructor::Fun(ld, lr), Constructor::Fun(rd, rr)) => {
                ld.union(rd);
                lr.union(rr);
            }
            (Constructor::Record(ref mut lhs), Constructor::Record(ref rhs)) => match pol {
                Polarity::Pos => {
                    *lhs = lhs.clone().intersection_with(rhs.clone(), |mut l, r| {
                        l.union(&r);
                        l
                    })
                }
                Polarity::Neg => {
                    *lhs = lhs.clone().union_with(rhs.clone(), |mut l, r| {
                        l.union(&r);
                        l
                    })
                }
            },
            _ => unreachable!(),
        }
    }

    fn visit_params_intersection<F, E>(&self, other: &Self, mut visit: F) -> Result<(), E>
    where
        F: FnMut(Self::Label, &StateSet, &StateSet) -> Result<(), E>,
    {
        itertools::merge_join_by(self.params(), other.params(), |l, r| Ord::cmp(&l.0, &r.0))
            .try_for_each(|eob| match eob {
                EitherOrBoth::Both(l, r) => visit(l.0, &l.1, &r.1),
                _ => Ok(()),
            })
    }

    fn map<F>(self, mut mapper: F) -> Self
    where
        F: FnMut(Self::Label, StateSet) -> StateSet,
    {
        match self {
            Constructor::Bool => Constructor::Bool,
            Constructor::Fun(d, r) => {
                Constructor::Fun(mapper(Label::Domain, d), mapper(Label::Range, r))
            }
            Constructor::Record(fields) => Constructor::Record(
                fields
                    .into_iter()
                    .map(|(label, set)| (label.clone(), mapper(Label::Label(label), set)))
                    .collect(),
            ),
        }
    }
}

impl Constructor {
    fn params(&self) -> Vec<(Label, StateSet)> {
        match self {
            Constructor::Bool => vec![],
            Constructor::Fun(d, r) => vec![(Label::Domain, d.clone()), (Label::Range, r.clone())],
            Constructor::Record(fields) => fields
                .clone()
                .into_iter()
                .map(|(label, set)| (Label::Label(label), set))
                .collect(),
        }
    }
}

impl PartialOrd for Constructor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Constructor::Bool, Constructor::Bool) => Some(Ordering::Equal),
            (Constructor::Fun(..), Constructor::Fun(..)) => Some(Ordering::Equal),
            (Constructor::Record(ref lhs), Constructor::Record(ref rhs)) => {
                iter_set::cmp(lhs.keys(), rhs.keys()).map(Ordering::reverse)
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Label {
    Domain,
    Range,
    Label(Rc<str>),
}

impl crate::Label for Label {
    fn polarity(&self) -> Polarity {
        match self {
            Label::Domain => Polarity::Neg,
            Label::Range | Label::Label(_) => Polarity::Pos,
        }
    }
}
