use std::collections::VecDeque;
use std::rc::Rc;

use lazy_static::lazy_static;
use proptest::collection::btree_map;
use proptest::prelude::*;
use proptest::prop_oneof;
use proptest::proptest;
use proptest::strategy::{LazyJust, NewTree, ValueTree};
use proptest::string::string_regex;
use proptest::test_runner::TestRunner;
use rand_distr::Exp1;

use super::{Constructed, Constructor};
use crate::auto::{Automaton, StateId};
use crate::polar::Ty;
use crate::Polarity;

pub fn arb_auto_ty(pol: Polarity) -> BoxedStrategy<(Automaton<Constructor>, StateId)> {
    arb_polar_ty(pol)
        .prop_map(move |ty| {
            let mut auto = Automaton::new();
            let mut builder = auto.builder();
            let id = builder.build_polar(pol, &ty);
            drop(builder);
            (auto, id)
        })
        .boxed()
}

pub fn arb_polar_ty(pol: Polarity) -> BoxedStrategy<Ty<Constructed, char>> {
    prop_oneof![
        LazyJust::new(|| Ty::Zero),
        prop::char::range('a', 'e').prop_map(Ty::UnboundVar),
        BoundVar.prop_map(Ty::BoundVar),
    ]
    .prop_recursive(32, 1000, 8, |inner| {
        prop_oneof![
            3 => arb_polar_cons(inner.clone()).prop_map(Ty::Constructed),
            1 => (inner.clone(), inner.clone()).prop_map(|(l, r)| Ty::Add(Box::new(l), Box::new(r))),
            1 => inner.prop_map(Box::new).prop_map(Ty::Recursive),
        ]
    })
    .prop_filter("invalid polar type", move |ty| check(pol, ty, &mut VecDeque::new(), 0))
    .boxed()
}

fn arb_polar_cons(ty: BoxedStrategy<Ty<Constructed, char>>) -> BoxedStrategy<Constructed> {
    lazy_static! {
        static ref IDENT: SBoxedStrategy<Rc<str>> =
            string_regex("[a-z]").unwrap().prop_map(Into::into).sboxed();
    }

    prop_oneof![
        LazyJust::new(|| Constructed::Bool),
        (ty.clone(), ty.clone()).prop_map(|(d, r)| Constructed::Fun(Box::new(d), Box::new(r))),
        btree_map(IDENT.clone(), ty.prop_map(Box::new), 0..8).prop_map(Constructed::Record)
    ]
    .boxed()
}

fn check(
    pol: Polarity,
    ty: &Ty<Constructed, char>,
    recs: &mut VecDeque<Polarity>,
    unguarded: usize,
) -> bool {
    match ty {
        Ty::BoundVar(idx) => {
            if *idx < unguarded || *idx >= recs.len() {
                false
            } else {
                recs[*idx] == pol
            }
        }
        Ty::Constructed(Constructed::Fun(d, r)) => {
            check(-pol, d, recs, 0) && check(pol, r, recs, 0)
        }
        Ty::Constructed(Constructed::Record(fields)) => {
            fields.iter().all(|(_, t)| check(pol, t, recs, 0))
        }
        Ty::Add(l, r) => check(pol, l, recs, unguarded) && check(pol, r, recs, unguarded),
        Ty::Recursive(t) => {
            recs.push_front(pol);
            let b = check(pol, t, recs, unguarded + 1);
            recs.pop_front();
            b
        }
        _ => true,
    }
}

#[derive(Debug)]
struct BoundVar;
struct BoundVarTree(usize);

impl Strategy for BoundVar {
    type Tree = BoundVarTree;
    type Value = usize;

    fn new_tree(&self, runner: &mut TestRunner) -> NewTree<Self> {
        let val: f64 = runner.rng().sample(Exp1);
        Ok(BoundVarTree(val as usize))
    }
}

impl ValueTree for BoundVarTree {
    type Value = usize;

    fn current(&self) -> Self::Value {
        self.0
    }

    fn simplify(&mut self) -> bool {
        false
    }

    fn complicate(&mut self) -> bool {
        false
    }
}

proptest! {
    #[test]
    fn polar_pos(_ in arb_polar_ty(Polarity::Pos)) {}

    #[test]
    fn polar_neg(_ in arb_polar_ty(Polarity::Neg)) {}

    #[test]
    fn auto_pos(_ in arb_auto_ty(Polarity::Pos)) {}

    #[test]
    fn auto_neg(_ in arb_auto_ty(Polarity::Neg)) {}
}
