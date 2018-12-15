use std::cmp::Ordering;
use std::rc::Rc;
use std::mem::{Discriminant, discriminant};

use im::OrdSet;
use mlsub::*;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Symbol {
    Domain,
    Range,
    Label(Rc<str>),
}

impl trans::Symbol for Symbol {
    fn polarity(&self) -> Polarity {
        match self {
            Symbol::Domain => Polarity::Neg,
            Symbol::Range | Symbol::Label(_) => Polarity::Pos,
        }
    }
}

#[derive(Clone)]
pub enum Constructor {
    Bool,
    Fun,
    Record(OrdSet<Rc<str>>),
}

impl cons::Constructor for Constructor {
    type Key = Discriminant<Self>;

    fn key(&self) -> Self::Key {
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

pub struct MlSub;

impl TypeSystem for MlSub {
    type Constructor = Constructor;
    type Symbol = Symbol;
}
