#[allow(dead_code)]
mod common;

use proptest::{proptest, proptest_helper};
use mlsub::Polarity;
use mlsub::auto::Automaton;

use crate::common::{arb_auto_ty, arb_polar_ty};

proptest! {
    #[test]
    fn reduce_pos((nfa, nfa_start) in arb_auto_ty(Polarity::Pos)) {
        let mut dfa = Automaton::new();
        dfa.reduce(Polarity::Pos, &nfa, nfa_start);
    }

    #[test]
    #[ignore]
    fn reduce_posp(ty in arb_polar_ty(Polarity::Pos)) {
        let mut builder = Automaton::builder();
        let nfa_start = builder.build_polar(Polarity::Pos, &ty);
        let nfa = builder.build();

        let mut dfa = Automaton::new();
        dfa.reduce(Polarity::Pos, &nfa, nfa_start);
    }
}