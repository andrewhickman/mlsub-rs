use std::collections::HashSet;

use crate::biunify::bisubst::{bisubst, fixpoint, subst, Bisubst};
use crate::polar::Ty;
use crate::tests::Constructed;
use crate::Polarity;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Constraint(Ty<Constructed, char>, Ty<Constructed, char>);

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

pub(crate) fn biunify(
    lhs: Ty<Constructed, char>,
    rhs: Ty<Constructed, char>,
) -> Result<Bisubst, ()> {
    let mut cons = vec![Constraint(lhs, rhs)];
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
