#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::mem::replace;

use im::{hashset, HashSet};
use itertools::Itertools;

use crate::auto::{
    Automaton, FlowSet, State, StateId, StateRange, Symbol, Transition, TransitionSet,
};
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
    pub fn reduce<I>(&mut self, nfa: &Self, nfa_ids: I) -> StateRange
    where
        I: IntoIterator<Item = (StateId, Polarity)>,
    {
        self.states.reserve(nfa.states.len());

        // Maps between sets of nfa states to corresponding dfa state.
        let mut map = BiMap::with_capacity(nfa.states.len());

        let start = self.next();
        // Stack of states to be converted from nfa states to dfa states.
        let mut stack: Vec<_> = nfa_ids
            .into_iter()
            .map(|(nfa_id, pol)| {
                #[cfg(debug_assertions)]
                debug_assert_eq!(nfa.index(nfa_id).pol, pol);

                let dfa_id = self.add(nfa.index(nfa_id).clone());
                map.insert(hashset![nfa_id], dfa_id);
                (dfa_id, pol)
            })
            .collect();
        let range = self.range_from(start);

        debug_assert!(stack.iter().map(|&(id, _)| id).eq(range.clone()));

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

        range
    }
}

struct BiMap {
    // maps nfa state set to corresponding dfa state
    ns2d: HashMap<HashSet<StateId>, StateId>,
    // maps nfa state to set of dfa states containing it
    n2ds: HashMap<StateId, HashSet<StateId>>,
}

impl BiMap {
    fn with_capacity(cap: usize) -> Self {
        BiMap {
            ns2d: HashMap::with_capacity(cap),
            n2ds: HashMap::with_capacity(cap),
        }
    }

    fn insert(&mut self, ns: HashSet<StateId>, d: StateId) {
        self.ns2d.insert(ns.clone(), d);
        for n in ns {
            self.n2ds.entry(n).or_default().insert(d);
        }
    }
}
