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

#[test]
fn reduce_x() {
    use std::collections::HashMap;

    use crate::auto::Automaton;
    use crate::polar::Ty;
    use crate::tests::Constructed;

    let polar = Ty::Add::<Constructed, char>(
        Box::new(Ty::Constructed(Constructed::Record(HashMap::new()))),
        Box::new(Ty::Constructed(Constructed::Fun(
            Box::new(Ty::UnboundVar('a')),
            Box::new(Ty::UnboundVar('a')),
        ))),
    );

    let mut builder = Automaton::builder();
    let nfa_start = builder.build_polar(Polarity::Pos, &polar);
    let nfa = builder.build();

    let mut dfa = Automaton::new();
    dfa.reduce(Polarity::Pos, &nfa, nfa_start);
}
