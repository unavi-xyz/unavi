use std::collections::BTreeMap;

use smol_str::SmolStr;

use crate::{
    attributes::parent::ParentAttr,
    property::Property,
    state::entry::Stamp,
};

/// One prim as every reader sees it: each key composed from the layer stack,
/// strongest opinion winning.
///
/// A cache rather than a source — only `HsdState`'s resolve step writes it,
/// and it holds nothing the layers do not already say.
#[derive(Debug, Clone, Default)]
pub struct PrimState {
    /// `None` means no layer states a live parent: the prim has not been
    /// written yet, was tombstoned, or a stronger layer blocked it. Either way
    /// it is held rather than realized.
    pub parent:   Option<ParentAttr>,
    parent_stamp: Stamp,
    /// Attributes and relationships share one map; the [`Property`] variant
    /// says which.
    props:        BTreeMap<SmolStr, Property>,
}

impl PrimState {
    #[must_use]
    pub fn property(&self, name: &str) -> Option<&Property> {
        self.props.get(name)
    }

    pub fn properties(&self) -> impl Iterator<Item = (&SmolStr, &Property)> {
        self.props.iter()
    }

    /// The stamp of whichever layer won the parent key, which is what breaks a
    /// cycle identically on every peer.
    #[must_use]
    pub const fn parent_stamp(&self) -> Stamp {
        self.parent_stamp
    }

    pub(super) const fn set_parent(&mut self, parent: Option<ParentAttr>, stamp: Stamp) {
        self.parent = parent;
        self.parent_stamp = stamp;
    }

    pub(super) fn set_property(&mut self, name: &str, value: Option<Property>) {
        match value {
            Some(value) => {
                // Update in place where the key exists: `insert` would build a
                // fresh `SmolStr` and drop it, since a `BTreeMap` keeps the
                // key it already had.
                if let Some(slot) = self.props.get_mut(name) {
                    *slot = value;
                } else {
                    self.props.insert(SmolStr::new(name), value);
                }
            }
            None => {
                self.props.remove(name);
            }
        }
    }

    /// The stored key equal to `name`, so an event can clone the interned
    /// string instead of building a fresh one.
    pub(super) fn property_key(&self, name: &str) -> Option<SmolStr> {
        self.props.get_key_value(name).map(|(key, _)| key.clone())
    }
}
