use std::collections::HashMap;
use std::rc::Rc;

use mlsub::auto::{Build, Builder, StateId};
use mlsub::{polar, Polarity};

use super::{Constructor, MlSub, Symbol};

#[derive(Debug)]
pub enum Constructed {
    Bool,
    Fun(
        Box<polar::Ty<Constructed, char>>,
        Box<polar::Ty<Constructed, char>>,
    ),
    Record(HashMap<Rc<str>, Box<polar::Ty<Constructed, char>>>),
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
