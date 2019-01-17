#![allow(unused)]

use std::cmp::Ordering;

use crate::auto::{Automaton, StateId};
use crate::TypeSystem;

pub struct Ty<'a, T: TypeSystem> {
    pub auto: &'a Automaton<T>,
    pub id: StateId,
}

impl<'a, T: TypeSystem> PartialOrd for Ty<'a, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        assert!(self.auto.is_reduced());
        assert!(other.auto.is_reduced());

        unimplemented!()
    }
}

impl<'a, T: TypeSystem> PartialEq for Ty<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}
