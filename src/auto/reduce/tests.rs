use proptest::{proptest, proptest_helper};

use crate::auto::Automaton;
use crate::tests::arb_auto_ty;
use crate::Polarity;

proptest! {
    #[test]
    fn reduce_pos((nfa, nfa_start) in arb_auto_ty(Polarity::Pos)) {
        let mut dfa = Automaton::new();
        dfa.reduce(Polarity::Pos, &nfa, nfa_start);
    }
}
