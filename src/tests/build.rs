use std::collections::BTreeMap;
use std::rc::Rc;

use super::{Constructor, Label};
use crate::auto::StateSet;
use crate::auto::build::polar::Build;
use crate::polar::Ty;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constructed {
    Bool,
    Fun(Box<Ty<Constructed, char>>, Box<Ty<Constructed, char>>),
    Record(BTreeMap<Rc<str>, Box<Ty<Constructed, char>>>),
}

impl Build<Constructor, char> for Constructed {
    fn map<'a, F>(&'a self, mut mapper: F) -> Constructor
    where
        F: FnMut(Label, &'a Ty<Self, char>) -> StateSet,
    {
        match self {
            Constructed::Bool => Constructor::Bool,
            Constructed::Fun(lhs, rhs) => {
                Constructor::Fun(mapper(Label::Domain, lhs), mapper(Label::Range, rhs))
            }
            Constructed::Record(fields) => Constructor::Record(
                fields
                    .iter()
                    .map(|(label, ty)| (label.clone(), mapper(Label::Label(label.clone()), ty)))
                    .collect(),
            ),
        }
    }
}
