use std::collections::BTreeMap;

use crate::{
    attributes::parent::ParentAttr,
    property::{
        name::PropName,
        value::Value,
    },
    state::entry::Stamp,
};

/// One prim with each key composed from the layer stack, strongest opinion
/// winning.
#[derive(Debug, Clone, Default)]
pub struct PrimState {
    /// `None` when no layer states a live parent.
    pub parent:   Option<ParentAttr>,
    parent_stamp: Stamp,
    props:        BTreeMap<PropName, Value>,
}

impl PrimState {
    #[must_use]
    pub fn property(&self, name: &PropName) -> Option<&Value> {
        self.props.get(name)
    }

    pub fn properties(&self) -> impl Iterator<Item = (&PropName, &Value)> {
        self.props.iter()
    }

    /// The stamp of the opinion that won the parent key.
    #[must_use]
    pub const fn parent_stamp(&self) -> Stamp {
        self.parent_stamp
    }

    pub(super) const fn set_parent(&mut self, parent: Option<ParentAttr>, stamp: Stamp) {
        self.parent = parent;
        self.parent_stamp = stamp;
    }

    pub(super) fn set_property(&mut self, name: &PropName, value: Option<Value>) {
        match value {
            Some(value) => {
                if let Some(slot) = self.props.get_mut(name) {
                    *slot = value;
                } else {
                    self.props.insert(name.clone(), value);
                }
            }
            None => {
                self.props.remove(name);
            }
        }
    }
}
