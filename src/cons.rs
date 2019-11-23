use std::borrow::Cow;
use std::fmt::{self, Debug};

use itertools::{merge_join_by, EitherOrBoth};
use small_ord_set::{self, KeyValuePair, SmallOrdSet};

use crate::auto::StateSet;
use crate::Polarity;

pub trait Constructor: Clone + PartialOrd {
    type Component: Ord + Clone;
    type Label: Label;

    fn component(&self) -> Self::Component;
    fn join(&mut self, other: &Self, pol: Polarity);

    /// Visit the common type parameters of two constructors.
    fn visit_params_intersection<F, E>(&self, other: &Self, visit: F) -> Result<(), E>
    where
        F: FnMut(Self::Label, &StateSet, &StateSet) -> Result<(), E>;

    fn map<F>(self, mapper: F) -> Self
    where
        F: FnMut(Self::Label, StateSet) -> StateSet;
}

pub trait Label {
    fn polarity(&self) -> Polarity;
}

#[derive(Clone)]
pub struct ConstructorSet<C: Constructor> {
    set: SmallOrdSet<[KeyValuePair<C::Component, C>; 1]>,
}

impl<C: Constructor> ConstructorSet<C> {
    pub fn iter(&self) -> impl Iterator<Item = &C> + Clone {
        self.set.values()
    }

    pub(crate) fn add(&mut self, pol: Polarity, con: Cow<C>) {
        match self.set.entry(con.component()) {
            small_ord_set::Entry::Occupied(mut entry) => entry.get_mut().join(&con, pol),
            small_ord_set::Entry::Vacant(entry) => {
                entry.insert(con.into_owned());
            }
        }
    }

    pub(crate) fn intersection<'a>(
        &'a self,
        other: &'a Self,
    ) -> impl Iterator<Item = (&'a C, &'a C)> {
        merge_join_by(&self.set, &other.set, Ord::cmp).filter_map(|eob| match eob {
            EitherOrBoth::Both(l, r) => Some((&l.value, &r.value)),
            _ => None,
        })
    }

    pub(crate) fn merge(&mut self, other: &Self, pol: Polarity) {
        for con in other.iter() {
            self.add(pol, Cow::Borrowed(con));
        }
    }

    pub(crate) fn get(&self, cpt: C::Component) -> Option<&C> {
        self.set.get_value(&cpt)
    }

    pub(crate) fn shift(self, offset: usize) -> Self {
        let set = self
            .set
            .into_iter()
            .map(|kvp| KeyValuePair {
                key: kvp.key,
                value: kvp.value.map(|_, set| set.shift(offset)),
            })
            .collect();
        ConstructorSet { set }
    }
}

impl<C: Debug + Constructor> Debug for ConstructorSet<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<C: Constructor> Default for ConstructorSet<C> {
    fn default() -> Self {
        ConstructorSet {
            set: SmallOrdSet::default(),
        }
    }
}
