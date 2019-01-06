mod common;

use mlsub::Polarity;
use proptest::{proptest, proptest_helper};

use crate::common::{arb_polar_ty, arb_auto_ty};

proptest! {
    #[test]
    fn polar_pos(_ in arb_polar_ty(Polarity::Pos)) {}

    #[test]
    fn auto_pos(_ in arb_auto_ty(Polarity::Pos)) {}

    #[test]
    fn polar_neg(_ in arb_polar_ty(Polarity::Neg)) {}

    #[test]
    fn auto_neg(_ in arb_auto_ty(Polarity::Neg)) {}
}
