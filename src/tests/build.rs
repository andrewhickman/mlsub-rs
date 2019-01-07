use std::collections::BTreeMap;
use std::rc::Rc;

use super::{Constructor, MlSub, Symbol};
use crate::auto::{Build, Builder, StateId};
use crate::polar::Ty;
use crate::Polarity;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constructed {
    Bool,
    Fun(Box<Ty<Constructed, char>>, Box<Ty<Constructed, char>>),
    Record(BTreeMap<Rc<str>, Box<Ty<Constructed, char>>>),
}

impl Build<MlSub, char> for Constructed {
    fn build(&self, builder: &mut Builder<MlSub, char>, pol: Polarity) -> StateId {
        match self {
            Constructed::Bool => builder.build_constructor(pol, Constructor::Bool),
            Constructed::Fun(domain, range) => {
                let id = builder.build_constructor(pol, Constructor::Fun);
                builder.build_transition(pol, id, Symbol::Domain, domain);
                builder.build_transition(pol, id, Symbol::Range, range);
                id
            }
            Constructed::Record(fields) => {
                let keys = fields.keys().cloned().collect();
                let id = builder.build_constructor(pol, Constructor::Record(keys));
                for (label, ty) in fields {
                    builder.build_transition(pol, id, Symbol::Label(label.clone()), ty);
                }
                id
            }
        }
    }
}
