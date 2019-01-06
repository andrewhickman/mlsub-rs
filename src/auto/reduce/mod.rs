#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::mem::replace;

use im::{hashset, HashSet};
use itertools::Itertools;

use crate::auto::{Automaton, FlowSet, State, StateId, Symbol, Transition, TransitionSet};
use crate::{Polarity, TypeSystem};

impl<T: TypeSystem> State<T> {
    fn merged<'a, I>(pol: Polarity, it: I) -> Self
    where
        T: 'a,
        I: IntoIterator<Item = &'a Self>,
    {
        it.into_iter().fold(State::new(pol), |mut l, r| {
            #[cfg(debug_assertions)]
            debug_assert_eq!(r.pol, pol);

            l.cons.merge(&r.cons, pol);
            l.trans.union(&r.trans);
            l.flow.union(&r.flow);
            l
        })
    }
}

impl<T: TypeSystem> Automaton<T> {
    pub fn reduce(&mut self, pol: Polarity, nfa: &Self, nfa_start: StateId) -> StateId {
        #[cfg(debug_assertions)]
        debug_assert_eq!(nfa.index(nfa_start).pol, pol);

        self.states.reserve(nfa.states.len());

        let dfa_start = self.add(nfa.index(nfa_start).clone());
        // Stack of states to be converted from nfa states to dfa states.
        let mut stack = vec![(dfa_start, pol)];

        // Maps between sets of nfa states to corresponding dfa state.
        let mut map = BiMap::new();
        map.insert(hashset![nfa_start], dfa_start);

        // Walk transitions and convert to dfa ids.
        while let Some((a, a_pol)) = stack.pop() {
            // Remove old nfa ids
            let nfa_trans = replace(&mut self.index_mut(a).trans, TransitionSet::default());

            let mut dfa_trans = TransitionSet::default();
            for (symbol, ids) in &nfa_trans.into_iter().group_by(Transition::symbol) {
                let ids = ids.map(|tr| tr.id()).collect();

                dfa_trans.add(
                    symbol.clone(),
                    if let Some(&b) = map.ns2d.get(&ids) {
                        b
                    } else {
                        let b_pol = a_pol * symbol.polarity();
                        let state = State::merged(b_pol, ids.iter().map(|&id| nfa.index(id)));
                        let b = self.add(state);
                        map.insert(ids, b);
                        stack.push((b, b_pol));
                        b
                    },
                );
            }

            // Replace with dfa ids
            replace(&mut self.index_mut(a).trans, dfa_trans);
        }

        // Populate flow
        for &a in map.ns2d.values() {
            // Remove old nfa ids
            let nfa_flow = replace(&mut self.index_mut(a).flow, FlowSet::default());

            let dfa_flow = FlowSet::from_iter(
                nfa_flow
                    .iter()
                    .flat_map(|b| map.n2ds.get(&b).cloned())
                    .flatten(),
            );

            // Replace with dfa ids
            replace(&mut self.index_mut(a).flow, dfa_flow);
        }

        #[cfg(debug_assertions)]
        debug_assert!(self.check_flow());
        #[cfg(debug_assertions)]
        debug_assert!(self.is_reduced());

        dfa_start
    }
}

struct BiMap {
    // maps nfa state set to corresponding dfa state
    ns2d: HashMap<HashSet<StateId>, StateId>,
    // maps nfa state to set of dfa states containing it
    n2ds: HashMap<StateId, HashSet<StateId>>,
}

impl BiMap {
    fn new() -> Self {
        BiMap {
            ns2d: HashMap::new(),
            n2ds: HashMap::new(),
        }
    }

    fn insert(&mut self, ns: HashSet<StateId>, d: StateId) {
        self.ns2d.insert(ns.clone(), d);
        for n in ns {
            self.n2ds.entry(n).or_default().insert(d);
        }
    }
}
