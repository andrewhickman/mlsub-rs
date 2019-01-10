use std::collections::BTreeMap;
use std::rc::Rc;

use super::{Constructor, MlSub, Symbol};
use crate::auto::Build;
use crate::polar::Ty;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constructed {
    Bool,
    Fun(Box<Ty<Constructed, char>>, Box<Ty<Constructed, char>>),
    Record(BTreeMap<Rc<str>, Box<Ty<Constructed, char>>>),
}

impl Build<MlSub, char> for Constructed {
    fn constructor(&self) -> Constructor {
        match self {
            Constructed::Bool => Constructor::Bool,
            Constructed::Fun(..) => Constructor::Fun,
            Constructed::Record(fields) => Constructor::Record(fields.keys().cloned().collect()),
        }
    }

    fn visit_transitions<'a, F>(&'a self, mut visit: F)
    where
        F: FnMut(Symbol, &'a Ty<Constructed, char>),
    {
        match self {
            Constructed::Bool => (),
            Constructed::Fun(domain, range) => {
                visit(Symbol::Domain, &*domain);
                visit(Symbol::Range, &*range);
            }
            Constructed::Record(fields) => {
                for (label, ty) in fields {
                    visit(Symbol::Label(label.clone()), &*ty);
                }
            }
        }
    }
}
