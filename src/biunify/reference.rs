use std::collections::HashSet;
use std::ops;

use im::Vector;
use proptest::strategy::Strategy;

use crate::polar::Ty;
use crate::tests::{arb_polar_ty, Constructed};
use crate::Polarity;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::biunify) struct Constraint(
    pub(in crate::biunify) Ty<Constructed, char>,
    pub(in crate::biunify) Ty<Constructed, char>,
);

pub(in crate::biunify) fn arb_constraint() -> impl Strategy<Value = Constraint> {
    (arb_polar_ty(Polarity::Pos), arb_polar_ty(Polarity::Neg)).prop_map(|(l, r)| Constraint(l, r))
}

impl Constraint {
    fn bisubst(self, sub: &Bisubst) -> Self {
        Constraint(
            sub.apply(self.0, Polarity::Pos),
            sub.apply(self.1, Polarity::Neg),
        )
    }
}

fn subi(con: Constraint) -> Result<Vec<Constraint>, ()> {
    match con {
        Constraint(
            Ty::Constructed(Constructed::Fun(d1, r1)),
            Ty::Constructed(Constructed::Fun(d2, r2)),
        ) => Ok(vec![Constraint(*d2, *d1), Constraint(*r1, *r2)]),
        Constraint(Ty::Constructed(Constructed::Bool), Ty::Constructed(Constructed::Bool)) => {
            Ok(vec![])
        }
        Constraint(
            Ty::Constructed(Constructed::Record(f1)),
            Ty::Constructed(Constructed::Record(f2)),
        ) => {
            if iter_set::difference(f2.keys(), f1.keys()).next().is_none() {
                Ok(iter_set::intersection(f1.keys(), f2.keys())
                    .map(|key| Constraint(*f1[key].clone(), *f2[key].clone()))
                    .collect())
            } else {
                Err(())
            }
        }
        Constraint(Ty::Recursive(lhs), rhs) => {
            let lhs = subst((*lhs).clone(), 0, Ty::Recursive(lhs));
            Ok(vec![Constraint(lhs, rhs)])
        }
        Constraint(lhs, Ty::Recursive(rhs)) => {
            let rhs = subst((*rhs).clone(), 0, Ty::Recursive(rhs));
            Ok(vec![Constraint(lhs, rhs)])
        }
        Constraint(Ty::Add(lhsa, lhsb), rhs) => {
            Ok(vec![Constraint(*lhsa, rhs.clone()), Constraint(*lhsb, rhs)])
        }
        Constraint(lhs, Ty::Add(rhsa, rhsb)) => {
            Ok(vec![Constraint(lhs.clone(), *rhsa), Constraint(lhs, *rhsb)])
        }
        Constraint(Ty::Zero, _) => Ok(vec![]),
        Constraint(_, Ty::Zero) => Ok(vec![]),
        _ => Err(()),
    }
}

fn atomic(con: &Constraint) -> Result<Bisubst, ()> {
    match con {
        &Constraint(Ty::UnboundVar(v), Ty::Constructed(_))
        | &Constraint(Ty::UnboundVar(v), Ty::UnboundVar(_)) => Ok(Bisubst::unit(
            v,
            Polarity::Neg,
            fixpoint(Ty::Add(
                Box::new(Ty::UnboundVar(v)),
                Box::new(bisubst(
                    con.1.clone(),
                    Polarity::Neg,
                    (Polarity::Neg, v),
                    Ty::BoundVar(0),
                )),
            )),
        )),
        &Constraint(Ty::Constructed(_), Ty::UnboundVar(v)) => Ok(Bisubst::unit(
            v,
            Polarity::Pos,
            fixpoint(Ty::Add(
                Box::new(Ty::UnboundVar(v)),
                Box::new(bisubst(
                    con.0.clone(),
                    Polarity::Pos,
                    (Polarity::Pos, v),
                    Ty::BoundVar(0),
                )),
            )),
        )),
        _ => Err(()),
    }
}

#[derive(Debug, Clone)]
pub(in crate::biunify) struct Bisubst {
    sub: Vector<((Polarity, char), Ty<Constructed, char>)>,
}

impl Bisubst {
    fn new() -> Self {
        Bisubst { sub: Vector::new() }
    }

    fn unit(v: char, pol: Polarity, ty: Ty<Constructed, char>) -> Self {
        Bisubst {
            sub: Vector::unit(((pol, v), ty)),
        }
    }

    fn apply(&self, mut ty: Ty<Constructed, char>, pol: Polarity) -> Ty<Constructed, char> {
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

fn subst(
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

fn bisubst(
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

pub(in crate::biunify) fn biunify(constraint: Constraint) -> Result<Bisubst, ()> {
    biunify_all(vec![constraint])
}

pub(in crate::biunify) fn biunify_all(mut cons: Vec<Constraint>) -> Result<Bisubst, ()> {
    let mut hyp = HashSet::new();
    let mut result = Bisubst::new();
    while let Some(con) = cons.pop() {
        if hyp.contains(&con) {
            continue;
        } else if let Ok(bisub) = atomic(&con) {
            hyp.insert(con);
            cons = cons.into_iter().map(|con| con.bisubst(&bisub)).collect();
            hyp = hyp.into_iter().map(|con| con.bisubst(&bisub)).collect();
            result *= bisub;
        } else if let Ok(sub) = subi(con.clone()) {
            hyp.insert(con);
            cons.extend(sub);
        } else {
            return Err(());
        }
    }

    Ok(result)
}
