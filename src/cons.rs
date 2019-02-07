use std::borrow::Cow;
use std::hash::{BuildHasherDefault, Hash};
use std::iter::Flatten;
use std::{fmt, option};

use im::hashmap::{Entry, HashMap, Values};
use itertools::{merge_join_by, EitherOrBoth};
use lazy_static::lazy_static;
use seahash::SeaHasher;

use crate::auto::StateSet;
use crate::Polarity;

pub trait Constructor: Clone + PartialOrd {
    type Component: Eq + Hash + Clone;
    type Label: Label;
    type Params: Iterator<Item = (Self::Label, StateSet)>;

    fn component(&self) -> Self::Component;
    fn join(&mut self, other: &Self, pol: Polarity);

    /// Return the type parameters of this constructor, sorted according to the `Ord` impl of
    /// their label.
    fn params(&self) -> Self::Params;

    fn map<F>(self, mapper: F) -> Self
    where
        F: FnMut(Self::Label, StateSet) -> StateSet;
}

pub trait Label: Clone + Ord {
    fn polarity(&self) -> Polarity;
}

#[derive(Clone)]
pub(crate) struct ConstructorSet<C: Constructor> {
    set: Option<HashMap<C::Component, C, BuildHasherDefault<SeaHasher>>>,
}

impl<C: Constructor> ConstructorSet<C> {
    pub(crate) fn add(&mut self, pol: Polarity, con: Cow<C>) {
        match self.set().entry(con.component()) {
            Entry::Occupied(mut entry) => entry.get_mut().join(&con, pol),
            Entry::Vacant(entry) => {
                entry.insert(con.into_owned());
            }
        }
    }

    pub(crate) fn intersection(
        self,
        other: Self,
    ) -> impl Iterator<Item = (C::Label, StateSet, StateSet)> {
        // TODO horrible
        match (self.set, other.set) {
            (Some(lhs), Some(rhs)) => Some(
                lhs.intersection_with(rhs, |lc, rc| {
                    merge_join_by(lc.params(), rc.params(), |l, r| Ord::cmp(&l.0, &r.0))
                        .flat_map(|eob| match eob {
                            EitherOrBoth::Both(lc, rc) => Some((lc.0, lc.1, rc.1)),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                })
                .into_iter()
                .flat_map(|(_, v)| v),
            )
            .into_iter()
            .flatten(),
            _ => None.into_iter().flatten(),
        }
    }

    pub(crate) fn merge(&mut self, other: &Self, pol: Polarity) {
        for con in other {
            self.add(pol, Cow::Borrowed(con));
        }
    }

    #[cfg(debug_assertions)]
    pub(crate) fn is_reduced(&self) -> bool {
        self.into_iter()
            .all(|con| con.params().all(|(_, ids)| ids.is_reduced()))
    }

    fn set(&mut self) -> &mut HashMap<C::Component, C, BuildHasherDefault<SeaHasher>> {
        lazy_static! {
            static ref HASHER: HashMap<(), (), BuildHasherDefault<SeaHasher>> = HashMap::default();
        }

        self.set.get_or_insert_with(|| HASHER.new_from())
    }
}

impl<C: fmt::Debug + Constructor> fmt::Debug for ConstructorSet<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self).finish()
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
