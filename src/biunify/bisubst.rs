use std::ops;

use im::Vector;

use crate::polar::Ty;
use crate::tests::Constructed;
use crate::Polarity;

#[derive(Debug, Clone)]
pub(crate) struct Bisubst {
    sub: Vector<((Polarity, char), Ty<Constructed, char>)>,
}

impl Bisubst {
    pub(crate) fn new() -> Self {
        Bisubst { sub: Vector::new() }
    }

    pub(crate) fn unit(v: char, pol: Polarity, ty: Ty<Constructed, char>) -> Self {
        Bisubst {
            sub: Vector::unit(((pol, v), ty)),
        }
    }

    pub(crate) fn apply(
        &self,
        mut ty: Ty<Constructed, char>,
        pol: Polarity,
    ) -> Ty<Constructed, char> {
        for (v, sub) in &self.sub {
            ty = bisubst(ty, pol, *v, sub.clone())
        }
        ty
    }
}

impl ops::MulAssign for Bisubst {
    fn mul_assign(&mut self, other: Self) {
        self.sub.append(other.sub)
    }
}

pub(crate) fn subst(
    ty: Ty<Constructed, char>,
    var: usize,
    sub: Ty<Constructed, char>,
) -> Ty<Constructed, char> {
    match ty {
        Ty::Add(l, r) => Ty::Add(
            Box::new(subst(*l, var, sub.clone())),
            Box::new(subst(*r, var, sub)),
        ),
        Ty::Recursive(t) => Ty::Recursive(Box::new(subst(*t, var + 1, sub))),
        Ty::BoundVar(idx) if idx == var => sub,
        Ty::Constructed(Constructed::Fun(d, r)) => Ty::Constructed(Constructed::Fun(
            Box::new(subst(*d, var, sub.clone())),
            Box::new(subst(*r, var, sub)),
        )),
        Ty::Constructed(Constructed::Record(fields)) => Ty::Constructed(Constructed::Record(
            fields
                .into_iter()
                .map(|(k, v)| (k, Box::new(subst(*v, var, sub.clone()))))
                .collect(),
        )),
        _ => ty,
    }
}

pub(crate) fn bisubst(
    ty: Ty<Constructed, char>,
    pol: Polarity,
    var: (Polarity, char),
    mut sub: Ty<Constructed, char>,
) -> Ty<Constructed, char> {
    match ty {
        Ty::Add(l, r) => Ty::Add(
            Box::new(bisubst(*l, pol, var, sub.clone())),
            Box::new(bisubst(*r, pol, var, sub)),
        ),
        Ty::Recursive(t) => {
            shift(&mut sub, 1);
            Ty::Recursive(Box::new(bisubst(*t, pol, var, sub)))
        }
        Ty::UnboundVar(v) if (pol, v) == var => sub,
        Ty::Constructed(Constructed::Fun(d, r)) => Ty::Constructed(Constructed::Fun(
            Box::new(bisubst(*d, -pol, var, sub.clone())),
            Box::new(bisubst(*r, pol, var, sub)),
        )),
        Ty::Constructed(Constructed::Record(fields)) => Ty::Constructed(Constructed::Record(
            fields
                .into_iter()
                .map(|(k, v)| (k, Box::new(bisubst(*v, pol, var, sub.clone()))))
                .collect(),
        )),
        _ => ty,
    }
}

fn split(ty: Ty<Constructed, char>, var: usize) -> (Ty<Constructed, char>, Ty<Constructed, char>) {
    match ty {
        Ty::BoundVar(idx) if idx == var => (ty, Ty::Zero),
        Ty::Zero => (Ty::Zero, Ty::Zero),
        Ty::Add(l, r) => {
            let (la, lg) = split(*l, var);
            let (ra, rg) = split(*r, var);
            (
                Ty::Add(Box::new(la), Box::new(ra)),
                Ty::Add(Box::new(lg), Box::new(rg)),
            )
        }
        Ty::BoundVar(_) | Ty::UnboundVar(_) | Ty::Constructed(_) => (Ty::Zero, ty),
        Ty::Recursive(ref t) => {
            let (ta, tg) = split((**t).clone(), var + 1);
            (ta, subst(tg, var + 1, ty))
        }
    }
}

pub(crate) fn fixpoint(ty: Ty<Constructed, char>) -> Ty<Constructed, char> {
    Ty::Recursive(Box::new(split(ty, 0).1))
}

fn shift(ty: &mut Ty<Constructed, char>, n: usize) {
    match ty {
        Ty::BoundVar(idx) => *idx += n,
        Ty::Add(l, r) => {
            shift(l, n);
            shift(r, n);
        }
        Ty::Constructed(Constructed::Fun(d, r)) => {
            shift(d, n);
            shift(r, n);
        }
        Ty::Constructed(Constructed::Record(fields)) => {
            fields.values_mut().for_each(|t| shift(t, n))
        }
        Ty::Recursive(t) => shift(t, n),
        _ => (),
    }
}
