mod arbitrary;
mod build;

pub use self::arbitrary::{arb_auto_ty, arb_polar_ty};
pub use self::build::Constructed;

use std::cmp::Ordering;
use std::mem::{discriminant, Discriminant};
use std::rc::Rc;

use im::OrdSet;
use mlsub::{self, auto, Polarity, TypeSystem};

#[derive(Debug)]
pub struct MlSub;

impl TypeSystem for MlSub {
    type Constructor = Constructor;
    type Symbol = Symbol;
}

#[derive(Clone, Debug)]
pub enum Constructor {
    Bool,
    Fun,
    Record(OrdSet<Rc<str>>),
}

impl mlsub::Constructor for Constructor {
    type Component = Discriminant<Self>;

    fn component(&self) -> Self::Component {
        discriminant(self)
    }

    fn join(&mut self, other: &Self) {
        match (self, other) {
            (Constructor::Bool, Constructor::Bool) => (),
            (Constructor::Fun, Constructor::Fun) => (),
            (Constructor::Record(ref mut lhs), Constructor::Record(ref rhs)) => {
                *lhs = lhs.clone().intersection(rhs.clone())
            }
            _ => unreachable!(),
        }
    }

    fn meet(&mut self, other: &Self) {
        match (self, other) {
            (Constructor::Bool, Constructor::Bool) => (),
            (Constructor::Fun, Constructor::Fun) => (),
            (Constructor::Record(ref mut lhs), Constructor::Record(ref rhs)) => {
                *lhs = lhs.clone().union(rhs.clone())
            }
            _ => unreachable!(),
        }
    }
}

impl PartialOrd for Constructor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Constructor::Bool, Constructor::Bool) => Some(Ordering::Equal),
            (Constructor::Fun, Constructor::Fun) => Some(Ordering::Equal),
            (Constructor::Record(ref lhs), Constructor::Record(ref rhs)) => {
                iter_set::cmp(lhs, rhs).map(Ordering::reverse)
            }
            _ => None,
        }
    }
}

impl PartialEq for Constructor {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Symbol {
    Domain,
    Range,
    Label(Rc<str>),
}

impl auto::Symbol for Symbol {
    fn polarity(&self) -> Polarity {
        match self {
            Symbol::Domain => Polarity::Neg,
            Symbol::Range | Symbol::Label(_) => Polarity::Pos,
        }
    }
}
