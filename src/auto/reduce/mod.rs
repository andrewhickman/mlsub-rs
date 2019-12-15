#[cfg(test)]
mod tests;

use std::borrow::Cow;
use std::collections::HashMap;
use std::mem::replace;

use small_ord_set::SmallOrdSet;

use crate::auto::{Automaton, ConstructorSet, FlowSet, State, StateId, StateRange, StateSet};
use crate::{Constructor, Label, Polarity};

impl<C: Constructor> State<C> {
    fn merged<'a, I>(pol: Polarity, it: I) -> Self
    where
        C: 'a,
        I: IntoIterator<Item = &'a Self>,
    {
        it.into_iter().fold(State::new(pol), |mut l, r| {
            #[cfg(debug_assertions)]
            debug_assert_eq!(r.pol, pol);

            l.cons.merge(&r.cons, pol);
            l.flow.union(&r.flow);
            l
        })
    }
}

impl<C: Constructor> Automaton<C> {
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
                debug_assert_eq!(nfa[nfa_id].pol, pol);

                let dfa_id = self.add(nfa[nfa_id].clone());
                map.insert(vec![nfa_id], dfa_id);
                (dfa_id, pol)
            })
            .collect();
        let range = self.range_from(start);

        debug_assert!(stack.iter().map(|&(id, _)| id).eq(range.clone()));

        // Walk transitions and convert to dfa ids.
        while let Some((a, a_pol)) = stack.pop() {
            // Remove old nfa ids
            let nfa_cons = replace(&mut self[a].cons, ConstructorSet::default());

            let mut dfa_cons = ConstructorSet::default();
            for nfa_con in nfa_cons.iter() {
                let dfa_con = nfa_con.clone().map(|label, set| {
                    let mut ids: Vec<_> = set.iter().collect();
                    ids.sort();

                    if let Some(&b) = map.ns2d.get(&ids) {
                        StateSet::new(b)
                    } else {
                        let b_pol = a_pol * label.polarity();
                        let state = State::merged(b_pol, ids.iter().map(|&id| &nfa[id]));
                        let b = self.add(state);
                        map.insert(ids, b);
                        stack.push((b, b_pol));
                        StateSet::new(b)
                    }
                });

                dfa_cons.add(a_pol, Cow::Owned(dfa_con));
            }

            // Replace with dfa ids
            replace(&mut self[a].cons, dfa_cons);
        }

        // Populate flow
        for &a in map.ns2d.values() {
            // Remove old nfa ids
            let nfa_flow = replace(&mut self[a].flow, FlowSet::default());

            let dfa_flow = FlowSet::from_iter(
                nfa_flow
                    .iter()
                    .flat_map(|b| map.n2ds.get(&b).cloned())
                    .flatten(),
            );

            // Replace with dfa ids
            replace(&mut self[a].flow, dfa_flow);
        }

        #[cfg(debug_assertions)]
        debug_assert!(self.check_flow());

        range
    }
}

struct BiMap {
    // maps nfa state set to corresponding dfa state
    ns2d: HashMap<Vec<StateId>, StateId>,
    // maps nfa state to set of dfa states containing it
    n2ds: HashMap<StateId, SmallOrdSet<[StateId; 4]>>,
}

impl BiMap {
    fn with_capacity(cap: usize) -> Self {
        BiMap {
            ns2d: HashMap::with_capacity(cap),
            n2ds: HashMap::with_capacity(cap),
        }
    }

    fn insert(&mut self, ns: Vec<StateId>, d: StateId) {
        for &n in ns.iter() {
            self.n2ds.entry(n).or_default().insert(d);
        }
        self.ns2d.insert(ns, d);
    }
}
