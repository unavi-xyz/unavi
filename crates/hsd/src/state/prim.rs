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
                self.props.insert(SmolStr::new(name), value);
            }
            None => {
                self.props.remove(name);
            }
        }
    }
}
