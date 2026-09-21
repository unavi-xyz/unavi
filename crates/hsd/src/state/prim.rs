use std::collections::BTreeMap;

use smol_str::SmolStr;

use crate::{
    property::{
        Parent,
        Property,
    },
    state::entry::Stamp,
};

/// One prim as every reader sees it: each key composed from the layer stack,
/// strongest opinion winning.
///
/// A cache rather than a source — only `SceneState`'s resolve step writes it,
/// and it holds nothing the layers do not already say.
#[derive(Debug, Clone, Default)]
pub struct PrimState {
    /// `None` means no layer states a live parent: the prim has not been
    /// written yet, was tombstoned, or a stronger layer blocked it. Either way
    /// it is held rather than realized.
    pub parent:   Option<Parent>,
    parent_stamp: Stamp,
    props:        BTreeMap<SmolStr, Property>,
    slots:        BTreeMap<SmolStr, Vec<u8>>,
}

impl PrimState {
    #[must_use]
    pub fn property(&self, name: &str) -> Option<&Property> {
        self.props.get(name)
    }

    pub fn properties(&self) -> impl Iterator<Item = (&SmolStr, &Property)> {
        self.props.iter()
    }

    #[must_use]
    pub fn slot(&self, name: &str) -> Option<&[u8]> {
        self.slots.get(name).map(Vec::as_slice)
    }

    pub fn slots(&self) -> impl Iterator<Item = (&SmolStr, &[u8])> {
        self.slots
            .iter()
            .map(|(name, value)| (name, value.as_slice()))
    }

    /// The stamp of whichever layer won the parent key, which is what breaks a
    /// cycle identically on every peer.
    #[must_use]
    pub const fn parent_stamp(&self) -> Stamp {
        self.parent_stamp
    }

    pub(super) const fn set_parent(&mut self, parent: Option<Parent>, stamp: Stamp) {
        self.parent = parent;
        self.parent_stamp = stamp;
    }

    pub(super) fn set_property(&mut self, name: &str, value: Option<Property>) {
        match value {
            Some(value) => {
                self.props.insert(SmolStr::new(name), value);
            }
            None => {
                self.props.remove(name);
            }
        }
    }

    pub(super) fn set_slot(&mut self, name: &str, value: Option<Vec<u8>>) {
        match value {
            Some(value) => {
                self.slots.insert(SmolStr::new(name), value);
            }
            None => {
                self.slots.remove(name);
            }
        }
    }
}
