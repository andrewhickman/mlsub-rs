use itertools::Itertools;
use proptest::collection::vec;
use proptest::test_runner::Config;
use proptest::{prop_assert_eq, proptest};

use crate::auto::Automaton;
use crate::biunify::reference::{self, arb_constraint};
use crate::polar::Ty;
use crate::tests::Constructed;
use crate::Polarity;

#[test]
fn constructed() {
    let mut auto = Automaton::new();

    let mut builder = auto.builder();
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
    drop(builder);

    assert!(auto.biunify(lhs_id, rhs_id).is_err());
}

proptest! {
    #![proptest_config(Config {
        cases: 1024,
        timeout: 10000,
        ..Config::default()
    })]

    #[test]
    fn biunify(con in arb_constraint()) {
        let mut auto = Automaton::new();

        let mut builder = auto.builder();
        let lhs_id = builder.build_polar(Polarity::Pos, &con.0);
        let rhs_id = builder.build_polar(Polarity::Neg, &con.1);
        drop(builder);

        prop_assert_eq!(
            auto.biunify(lhs_id, rhs_id).is_ok(),
            reference::biunify(con).is_ok()
        );
    }

    #[test]
    fn biunify_reduced(con in arb_constraint()) {
        let mut auto = Automaton::new();

        let mut builder = auto.builder();
        let lhs_id = builder.build_polar(Polarity::Pos, &con.0);
        let rhs_id = builder.build_polar(Polarity::Neg, &con.1);
        drop(builder);

        let mut reduced = Automaton::new();
        let dfa_ids: Vec<_> = reduced.reduce(&auto, [(lhs_id, Polarity::Pos), (rhs_id, Polarity::Neg)].iter().cloned()).collect();

        prop_assert_eq!(
            reduced.biunify(dfa_ids[0], dfa_ids[1]).is_ok(),
            reference::biunify(con).is_ok()
        );
    }
}

proptest! {
    #![proptest_config(Config {
        cases: 256,
        timeout: 10000,
        ..Config::default()
    })]

    #[test]
    fn biunify_all(cons in vec(arb_constraint(), 0..16)) {
        let mut auto = Automaton::new();

        let mut builder = auto.builder();
        let ids: Vec<_> = cons.iter().map(|con| {
            let lhs_id = builder.build_polar(Polarity::Pos, &con.0);
            let rhs_id = builder.build_polar(Polarity::Neg, &con.1);
            (lhs_id, rhs_id)
        }).collect();
        drop(builder);

        prop_assert_eq!(
            auto.biunify_all(ids).is_ok(),
            reference::biunify_all(cons).is_ok()
        );
    }

    #[test]
    fn biunify_all_reduced(cons in vec(arb_constraint(), 0..16)) {
        let mut auto = Automaton::new();

        let mut builder = auto.builder();
        let ids: Vec<_> = cons.iter().flat_map(|con| {
            let lhs_id = builder.build_polar(Polarity::Pos, &con.0);
            let rhs_id = builder.build_polar(Polarity::Neg, &con.1);
            vec![(lhs_id, Polarity::Pos), (rhs_id, Polarity::Neg)]
        }).collect();
        drop(builder);

        let mut reduced = Automaton::new();
        let dfa_ids = reduced.reduce(&auto, ids);

        prop_assert_eq!(
            reduced.biunify_all(dfa_ids.tuples()).is_ok(),
            reference::biunify_all(cons).is_ok()
        );
    }
}
