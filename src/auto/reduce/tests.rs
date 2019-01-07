use std::iter::once;

use proptest::{proptest, proptest_helper};

use crate::auto::Automaton;
use crate::tests::arb_auto_ty;
use crate::Polarity;

proptest! {
    #[test]
    fn reduce_one_pos((nfa, nfa_start) in arb_auto_ty(Polarity::Pos)) {
        let mut dfa = Automaton::new();
        dfa.reduce(&nfa, once((nfa_start, Polarity::Pos)));
    }

    #[test]
    fn reduce_one_neg((nfa, nfa_start) in arb_auto_ty(Polarity::Neg)) {
        let mut dfa = Automaton::new();
        dfa.reduce(&nfa, once((nfa_start, Polarity::Neg)));
    }
}
