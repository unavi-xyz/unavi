use std::collections::HashSet;

use bytes::Bytes;

use crate::{
    id::PrimId,
    property::{
        Property,
        value::Value,
    },
    schema::{
        parent::ParentAttr,
        reference::LayerKey,
    },
    state::{
        HsdState,
        StateError,
        entry::Stamp,
        layer::{
            Layer,
            LayerId,
            OpinionKey,
        },
        opinion::Opinion,
    },
};

impl HsdState {
    /// This document's opinions about the prims of the document `site`
    /// references.
    #[must_use]
    pub fn reference_layer_for(&self, site: PrimId) -> Option<&Layer> {
        self.references.get(&site)
    }

    /// Replaces the override layer with what the referencing document states,
    /// recomposing every key either layer held. Later changes arrive through
    /// [`Self::project_override`].
    pub fn install_reference_layer(&mut self, layer: &Layer) {
        let mut keys = self
            .layer(LayerId::Override)
            .keys()
            .into_iter()
            .collect::<HashSet<_>>();
        keys.extend(layer.keys());
        *self.layer_mut(LayerId::Override) = layer.clone();

        for (prim, opinion) in keys {
            match opinion {
                OpinionKey::Parent => self.settle_parent(prim),
                OpinionKey::Property(name) => self.settle_property(prim, &name),
            }
        }
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
