use proptest::{prop_assert, prop_assert_eq, proptest, proptest_helper};

use crate::auto::Automaton;
use crate::biunify::reference;
use crate::polar::Ty;
use crate::tests::{arb_polar_ty, Constructed};
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
    builder.finish();

    assert!(!auto.biunify(lhs_id, rhs_id));
}

proptest! {
    #[test]
    fn biunify(lhs in arb_polar_ty(Polarity::Pos), rhs in arb_polar_ty(Polarity::Neg)) {
        let mut auto = Automaton::new();

        let mut builder = auto.builder();
        let lhs_id = builder.build_polar(Polarity::Pos, &lhs);
        let rhs_id = builder.build_polar(Polarity::Neg, &rhs);
        builder.finish();

        prop_assert_eq!(
            auto.biunify(lhs_id, rhs_id),
            reference::biunify(lhs, rhs).is_ok()
        );
    }

    #[test]
    fn biunify_reduced(lhs in arb_polar_ty(Polarity::Pos), rhs in arb_polar_ty(Polarity::Neg)) {
        let mut auto = Automaton::new();

        let mut builder = auto.builder();
        let lhs_id = builder.build_polar(Polarity::Pos, &lhs);
        let rhs_id = builder.build_polar(Polarity::Neg, &rhs);
        builder.finish();

        let mut reduced = Automaton::new();
        let dfa_ids = reduced.reduce(&auto, [(lhs_id, Polarity::Pos), (rhs_id, Polarity::Neg)].iter().cloned());

        prop_assert_eq!(
            reduced.biunify(dfa_ids.start, dfa_ids.start + 1),
            reference::biunify(lhs, rhs).is_ok()
        );
    }
}
