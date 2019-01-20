use std::borrow::Cow;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash};
use std::iter::Flatten;
use std::option;

use im::hashmap::{Entry, HashMap, Values};
use lazy_static::lazy_static;
use seahash::SeaHasher;

use crate::Polarity;

/// Defines elements of the type lattice.
pub trait Constructor: PartialOrd + Clone {
    type Component: Eq + Hash + Clone + Debug;

    fn component(&self) -> Self::Component;

    fn join(&mut self, other: &Self);
    fn meet(&mut self, other: &Self);
}

#[derive(Clone, Debug)]
pub(crate) struct ConstructorSet<C: Constructor> {
    set: Option<HashMap<C::Component, C, BuildHasherDefault<SeaHasher>>>,
}

impl<C: Constructor> ConstructorSet<C> {
    pub(crate) fn add_pos(&mut self, con: Cow<C>) {
        match self.set().entry(con.component()) {
            Entry::Occupied(mut entry) => entry.get_mut().join(&con),
            Entry::Vacant(entry) => {
                entry.insert(con.into_owned());
            }
        }
    }

    pub(crate) fn add_neg(&mut self, con: Cow<C>) {
        match self.set().entry(con.component()) {
            Entry::Occupied(mut entry) => entry.get_mut().meet(&con),
            Entry::Vacant(entry) => {
                entry.insert(con.into_owned());
            }
        }
    }

    pub(crate) fn join(&mut self, other: &Self) {
        for con in other {
            self.add_pos(Cow::Borrowed(con));
        }
    }

    pub(crate) fn meet(&mut self, other: &Self) {
        for con in other {
            self.add_neg(Cow::Borrowed(con));
        }
    }

    pub(crate) fn merge(&mut self, other: &Self, pol: Polarity) {
        match pol {
            Polarity::Pos => self.join(other),
            Polarity::Neg => self.meet(other),
        }
    }

    fn set(&mut self) -> &mut HashMap<C::Component, C, BuildHasherDefault<SeaHasher>> {
        lazy_static! {
            static ref HASHER: HashMap<(), (), BuildHasherDefault<SeaHasher>> = HashMap::default();
        }

        self.set.get_or_insert_with(|| HASHER.new_from())
    }
}

impl<C: Constructor> Default for ConstructorSet<C> {
    fn default() -> Self {
        ConstructorSet { set: None }
    }
}

impl<'a, C: Constructor> IntoIterator for &'a ConstructorSet<C> {
    type Item = &'a C;
    type IntoIter = Flatten<option::IntoIter<Values<'a, C::Component, C>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.set.as_ref().map(HashMap::values).into_iter().flatten()
    }
}
