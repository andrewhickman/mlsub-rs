use std::collections::HashSet;

use proptest::{prop_assert, prop_assert_eq, proptest, proptest_helper};

use crate::auto::Automaton;
use crate::polar::Ty;
use crate::tests::{arb_polar_ty, Constructed};
use crate::Polarity;

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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Constraint(Ty<Constructed, char>, Ty<Constructed, char>);

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

fn atomic(con: &Constraint) -> bool {
    match con {
        Constraint(Ty::UnboundVar(_), Ty::Constructed(_)) => true,
        Constraint(Ty::Constructed(_), Ty::UnboundVar(_)) => true,
        Constraint(Ty::UnboundVar(_), Ty::UnboundVar(_)) => true,
        _ => false,
    }
}

fn biunify_reference(mut cons: Vec<Constraint>) -> bool {
    let mut hyp = HashSet::new();
    while let Some(con) = cons.pop() {
        if hyp.contains(&con) {
            continue;
        } else if atomic(&con) {
            hyp.insert(con);
        } else if let Ok(sub) = subi(con.clone()) {
            hyp.insert(con);
            cons.extend(sub);
        } else {
            return false;
        }
    }
    true
}

#[test]
fn constructed() {
    let mut builder = Automaton::builder();

    let lhs_id = builder.build_polar(
        Polarity::Pos,
        &Ty::Constructed(Constructed::Record(Default::default())),
    );
    let rhs_id = builder.build_polar(
        Polarity::Neg,
        &Ty::Add(
            Box::new(Ty::Zero),
            Box::new(Ty::Constructed(Constructed::Bool)),
        ),
    );

    let mut auto = builder.build();

    assert!(!auto.biunify(lhs_id, rhs_id));
}

proptest! {
    #[test]
    fn biunify(lhs in arb_polar_ty(Polarity::Pos), rhs in arb_polar_ty(Polarity::Neg)) {
        let mut builder = Automaton::builder();

        let lhs_id = builder.build_polar(Polarity::Pos, &lhs);
        let rhs_id = builder.build_polar(Polarity::Neg, &rhs);

        let mut auto = builder.build();
        prop_assert_eq!(
            auto.biunify(lhs_id, rhs_id),
            biunify_reference(vec![Constraint(lhs, rhs)])
        );
    }

    #[test]
    fn biunify_reduced(lhs in arb_polar_ty(Polarity::Pos), rhs in arb_polar_ty(Polarity::Neg)) {
        let mut builder = Automaton::builder();

        let lhs_id = builder.build_polar(Polarity::Pos, &lhs);
        let rhs_id = builder.build_polar(Polarity::Neg, &rhs);
        let auto = builder.build();

        let mut reduced = Automaton::new();
        let dfa_ids = reduced.reduce(&auto, [(lhs_id, Polarity::Pos), (rhs_id, Polarity::Neg)].iter().cloned());

        prop_assert_eq!(
            reduced.biunify(dfa_ids.start, dfa_ids.start + 1),
            biunify_reference(vec![Constraint(lhs, rhs)])
        );
    }
}
