//! Reference layers: what this document says about the prims of the documents
//! it references, and what a document referencing *this* one says about its.

use std::collections::HashSet;

use crate::{
    attributes::{
        Attribute,
        parent::ParentAttr,
    },
    id::PrimId,
    property::Property,
    state::{
        HsdState,
        StateError,
        entry::Stamp,
        layer::{
            Layer,
            LayerId,
            OpinionKey,
        },
    },
};

impl HsdState {
    /// What this document says about the prims of the document `site`
    /// references, for the realizer to install into it.
    #[must_use]
    pub fn reference_layer_for(&self, site: PrimId) -> Option<&Layer> {
        self.references.get(&site)
    }

    /// Changes once per write to any of this document's reference layers, so a
    /// realizer holding a version knows whether what it installed is current.
    #[must_use]
    pub const fn references_version(&self) -> u64 {
        self.references_version
    }

    /// Installs what the document referencing this one says about its prims.
    ///
    /// Replaces the layer whole rather than merging: an opinion the referencing
    /// document dropped has to stop resolving, and only that document knows
    /// its own set. Every key either layer held is recomposed, so a dropped
    /// opinion falls back to what this document itself says.
    pub fn install_reference_layer(&mut self, layer: &Layer) {
        let mut keys: HashSet<_> = self.layer(LayerId::Override).keys().into_iter().collect();
        keys.extend(layer.keys());
        *self.layer(LayerId::Override) = layer.clone();

        for (prim, opinion) in keys {
            match opinion {
                OpinionKey::Parent => self.settle_parent(prim),
                OpinionKey::Property(name) => self.settle_property(prim, &name),
            }
        }
    }

    /// Records this document's opinion about a prim of the document `site`
    /// references. An empty value is an opinion too: it blocks the key, which
    /// is not the same as holding none.
    pub(super) fn write_reference(
        &mut self,
        site: PrimId,
        target: PrimId,
        name: &str,
        value: Option<&Vec<u8>>,
        stamp: Stamp,
    ) -> Result<(), StateError> {
        let layer = self.references.entry(site).or_default();
        let opinions = layer.entry(target);

        let accepted = match name {
            ParentAttr::KEY => {
                let parent = value
                    .map(|bytes| ParentAttr::from_wire(bytes))
                    .transpose()?
                    .flatten();
                opinions.set_parent(parent.into(), stamp)
            }
            name => {
                let property = value.map(|bytes| Property::decode(bytes)).transpose()?;
                opinions.set_property(name, property.into(), stamp)
            }
        };
        if accepted {
            self.references_version = self.references_version.wrapping_add(1);
        }
        Ok(())
    }
}
