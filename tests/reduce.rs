#[allow(dead_code)]
mod common;

use mlsub::auto::Automaton;
use mlsub::Polarity;
use proptest::{proptest, proptest_helper};

use crate::common::arb_auto_ty;

proptest! {
    #[test]
    fn reduce_pos((nfa, nfa_start) in arb_auto_ty(Polarity::Pos)) {
        let mut dfa = Automaton::new();
        dfa.reduce(Polarity::Pos, &nfa, nfa_start);
    }
}
