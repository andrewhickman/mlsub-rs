use std::collections::HashMap;
use std::iter::empty;
use std::mem::replace;
use std::rc::Rc;

use im::OrdMap;
use mlsub::auto::{Automaton, StateId};
use mlsub::{flow, Polarity};

use super::sys::*;

#[derive(Clone)]
pub enum Type {
    Zero,
    Bool,
    Add(Box<Type>, Box<Type>),
    Fun(Box<Type>, Box<Type>),
    Struct(OrdMap<Rc<str>, Type>),
    Recursive(Box<Type>),
    UnboundVar(Rc<str>),
    BoundVar(usize),
}

pub fn build_polar(auto: &mut Automaton<MlSub>, pol: Polarity, ty: &Type) -> StateId {
    let mut builder = Builder {
        recs: Vec::new(),
        vars: HashMap::new(),
        cur: None,
        auto,
    };
    let id = builder.build_polar(pol, ty);
    builder.finish();
    id
}

struct Builder<'a> {
    recs: Vec<StateId>,
    vars: HashMap<Rc<str>, (Vec<StateId>, Vec<StateId>)>,
    cur: Option<StateId>,
    auto: &'a mut Automaton<MlSub>,
}

impl Builder<'_> {
    fn set_current(&mut self, pol: Polarity) -> StateId {
        match self.cur {
            Some(id) => id,
            None => {
                let id = self.auto.build_empty(pol);
                self.cur = Some(id);
                id
            }
        }
    }

    fn build_polar(&mut self, pol: Polarity, ty: &Type) -> StateId {
        let cur = self.cur.take();
        self.build_polar_flat(pol, ty);
        replace(&mut self.cur, cur).unwrap()
    }

    fn build_polar_flat(&mut self, pol: Polarity, ty: &Type) {
        match ty {
            Type::Zero => self.build_zero(pol),
            Type::Bool => self.build_bool(pol),
            Type::Add(l, r) => self.build_add(pol, l, r),
            Type::Fun(d, r) => self.build_fun(pol, d, r),
            Type::Struct(fields) => self.build_struct(pol, fields),
            Type::Recursive(ty) => self.build_recursive(pol, ty),
            Type::UnboundVar(var) => self.build_unbound_var(pol, var),
            Type::BoundVar(var) => self.build_bound_var(pol, *var),
        }
    }

    fn build_zero(&mut self, pol: Polarity) {
        self.set_current(pol);
    }

    fn build_bool(&mut self, pol: Polarity) {
        let id = self.set_current(pol);
        self.auto.build_type(pol, id, Constructor::Bool, empty());
    }

    // TODO move to main crate somehow
    fn build_add(&mut self, pol: Polarity, l: &Type, r: &Type) {
        self.set_current(pol);
        self.build_polar_flat(pol, l);
        self.build_polar_flat(pol, r);
    }

    fn build_fun(&mut self, pol: Polarity, d: &Type, r: &Type) {
        let id = self.set_current(pol);
        let d = self.build_polar(-pol, d);
        let r = self.build_polar(pol, r);
        self.auto.build_type(
            pol,
            id,
            Constructor::Fun,
            [(Symbol::Domain, d), (Symbol::Range, r)].iter().cloned(),
        )
    }

    fn build_struct(&mut self, pol: Polarity, fields: &OrdMap<Rc<str>, Type>) {
        let id = self.set_current(pol);
        let fields: Vec<_> = fields
            .iter()
            .map(|(label, ty)| (Symbol::Label(label.clone()), self.build_polar(pol, ty)))
            .collect();
        self.auto.build_type(pol, id, Constructor::Fun, fields);
    }

    fn build_recursive(&mut self, pol: Polarity, ty: &Type) {
        let id = self.set_current(pol);
        self.recs.push(id);
        self.build_polar_flat(pol, ty);
        self.recs.pop();
    }

    fn build_unbound_var(&mut self, pol: Polarity, var: &Rc<str>) {
        let id = self.set_current(pol);
        let entry = self.vars.entry(var.clone()).or_default();
        match pol {
            Polarity::Neg => entry.0.push(id),
            Polarity::Pos => entry.1.push(id),
        }
    }

    fn build_bound_var(&mut self, pol: Polarity, var: usize) {
        let idx = self.recs.len() - var - 1;
        self.cur = Some(self.recs[idx]);
    }

    fn finish(&mut self) {
        for (_, (neg, pos)) in self.vars.drain() {
            self.auto.build_flow(neg.iter().cloned(), pos.iter().cloned())
        }
    }
}
