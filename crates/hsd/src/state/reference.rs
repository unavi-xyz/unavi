use bytes::Bytes;

use crate::{
    attributes::{
        parent::ParentAttr,
        reference::LayerKey,
    },
    id::PrimId,
    property::{
        Property,
        value::Value,
    },
    state::{
        HsdState,
        StateError,
        entry::Stamp,
        layer::Layer,
        opinion::Opinion,
    },
};

impl HsdState {
    /// This document's opinions about the prims of the document `site`
    /// references, which the referenced document stacks as an override layer.
    #[must_use]
    pub fn reference_layer_for(&self, site: PrimId) -> Option<&Layer> {
        self.references.get(&site)
    }

    /// Replaces this document's opinion on a key of the document `site`
    /// references. `None` blocks the key.
    pub(super) fn write_reference(
        &mut self,
        site: PrimId,
        LayerKey { target, name }: &LayerKey,
        value: Option<Bytes>,
        stamp: Stamp,
    ) -> Result<(), StateError> {
        let target = *target;
        let opinions = self.references.entry(site).or_default().entry(target);
        if *name == ParentAttr::NAME {
            let parent = match &value {
                Some(bytes) => ParentAttr::from_wire(bytes)?,
                None => None,
            };
            opinions.replace_parent(Opinion::from(parent), stamp);
        } else {
            let property = value.as_ref().map(Value::decode).transpose()?;
            opinions.replace_property(name, Opinion::from(property), stamp);
        }
        Ok(())
    }
}
