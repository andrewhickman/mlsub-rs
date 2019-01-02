use std::collections::HashMap;
use std::rc::Rc;

use mlsub::auto::{Build, Builder, StateId};
use mlsub::{polar, Polarity};

use super::{Constructor, MlSub, Symbol};

pub enum Constructed {
    Bool,
    Fun(
        Box<polar::Ty<Constructed, Rc<str>>>,
        Box<polar::Ty<Constructed, Rc<str>>>,
    ),
    Record(HashMap<Rc<str>, Constructed>),
}

impl Build<MlSub, Rc<str>> for Constructed {
    fn build(&self, builder: &mut Builder<MlSub, Rc<str>>, pol: Polarity) -> StateId {
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
