use std::collections::BTreeMap;

use smol_str::SmolStr;

use crate::{
    property::{
        Parent,
        Property,
    },
    state::entry::{
        BulkRef,
        Stamp,
    },
};

/// Where a prim came from, which is what decides whether saving writes it.
///
/// The gate spawning its frame geometry every session must not accumulate in
/// the home document; content loaded from a document or a prefab must.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    Document,
    Script,
}

#[derive(Debug, Clone)]
pub struct PrimState {
    /// `None` means no live `parent/` entry: the prim does not exist yet, or
    /// was tombstoned. Either way it is held rather than realized.
    pub parent:   Option<Parent>,
    pub origin:   Origin,
    parent_stamp: Stamp,
    props:        BTreeMap<SmolStr, (Property, Stamp)>,
    bulk:         BTreeMap<SmolStr, (BulkRef, Stamp)>,
}

impl PrimState {
    pub(super) const fn new(origin: Origin) -> Self {
        Self {
            parent: None,
            origin,
            parent_stamp: Stamp {
                timestamp: 0,
                content:   [0; 32],
            },
            props: BTreeMap::new(),
            bulk: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn property(&self, name: &str) -> Option<&Property> {
        self.props.get(name).map(|(value, _)| value)
    }

    pub fn properties(&self) -> impl Iterator<Item = (&SmolStr, &Property)> {
        self.props.iter().map(|(name, (value, _))| (name, value))
    }

    #[must_use]
    pub fn bulk(&self, slot: &str) -> Option<BulkRef> {
        self.bulk.get(slot).map(|(value, _)| *value)
    }

    pub fn bulk_slots(&self) -> impl Iterator<Item = (&SmolStr, BulkRef)> {
        self.bulk.iter().map(|(slot, (value, _))| (slot, *value))
    }

    #[must_use]
    pub const fn parent_stamp(&self) -> Stamp {
        self.parent_stamp
    }

    pub(super) fn set_parent(&mut self, parent: Option<Parent>, stamp: Stamp) -> bool {
        if stamp < self.parent_stamp {
            return false;
        }
        self.parent = parent;
        self.parent_stamp = stamp;
        true
    }

    pub(super) fn set_property(&mut self, name: &str, value: Property, stamp: Stamp) -> bool {
        if self.props.get(name).is_some_and(|(_, old)| stamp < *old) {
            return false;
        }
        self.props.insert(SmolStr::new(name), (value, stamp));
        true
    }

    pub(super) fn remove_property(&mut self, name: &str, stamp: Stamp) -> bool {
        if self.props.get(name).is_some_and(|(_, old)| stamp < *old) {
            return false;
        }
        self.props.remove(name).is_some()
    }

    pub(super) fn set_bulk(&mut self, slot: &str, value: BulkRef, stamp: Stamp) -> bool {
        if self.bulk.get(slot).is_some_and(|(_, old)| stamp < *old) {
            return false;
        }
        self.bulk.insert(SmolStr::new(slot), (value, stamp));
        true
    }

    pub(super) fn remove_bulk(&mut self, slot: &str, stamp: Stamp) -> bool {
        if self.bulk.get(slot).is_some_and(|(_, old)| stamp < *old) {
            return false;
        }
        self.bulk.remove(slot).is_some()
    }
}
