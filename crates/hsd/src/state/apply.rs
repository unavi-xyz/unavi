//! Applying entries, from the document or from a compiled package. Entries
//! arrive unordered — a child may be seen before its parent — so every path
//! here has to be order-independent.

use std::collections::{
    BTreeMap,
    HashSet,
};

use crate::{
    attributes::slots::is_slot_name,
    id::PrimId,
    key,
    meta::DocMeta,
    property::{
        Parent,
        Property,
    },
    state::{
        HsdState,
        StateError,
        entry::{
            Entry,
            Stamp,
        },
        layer::LayerId,
    },
};

impl HsdState {
    pub fn apply(&mut self, entry: &Entry) -> Result<(), StateError> {
        self.apply_at(LayerId::Document, entry)
    }

    /// Applies an entry to the session layer: what a present peer says this
    /// session.
    ///
    /// The same key space as a document entry and the same three-state
    /// opinion — an empty value blocks the key — but a stronger layer,
    /// replicated by state messages rather than by the document, and absent
    /// from the save set until a [`Self::commit`] promotes it.
    pub fn apply_session(&mut self, entry: &Entry) -> Result<(), StateError> {
        self.apply_at(LayerId::Session, entry)
    }

    fn apply_at(&mut self, layer: LayerId, entry: &Entry) -> Result<(), StateError> {
        let stamp = Stamp::new(entry.timestamp, &entry.value);
        let empty = entry.value.is_empty();

        match key::parse(&entry.key) {
            Some(key::Key::Meta) => {
                // The document's own metadata, never a peer's opinion.
                if !empty && layer == LayerId::Document {
                    self.meta = DocMeta::decode(&entry.value)?;
                }
            }
            Some(key::Key::Prop { prim, name }) if name == key::PARENT => {
                let parent = if empty {
                    None
                } else {
                    Some(Parent::decode(&entry.value)?)
                };
                self.write_parent(layer, prim, parent, Some(stamp));
            }
            Some(key::Key::Prop { prim, name }) if is_slot_name(&name) => {
                let value = if empty {
                    None
                } else {
                    Some(entry.value.clone())
                };
                self.write_slot(layer, prim, &name, value, stamp);
            }
            Some(key::Key::Prop { prim, name }) => {
                let value = if empty {
                    None
                } else {
                    Some(Property::decode(&entry.value)?)
                };
                self.write_property(layer, prim, &name, value, stamp);
            }
            // An override is durable in the document stating it, so it
            // arrives by sync and never as a session opinion.
            Some(key::Key::Override { site, target, name }) if layer == LayerId::Document => {
                self.write_override(site, target, &name, (!empty).then_some(&entry.value), stamp)?;
            }
            Some(key::Key::Override { .. }) | None => {}
        }
        Ok(())
    }

    /// Drops the session opinion on a key, so whatever the layers beneath it
    /// say composes again.
    ///
    /// Not the same as blocking the key: a block is an opinion, and this is
    /// the absence of one. What a peer leaving takes with it.
    pub fn clear_session(&mut self, prim: PrimId, name: &str) {
        let Some(layer) = self.layers.get_mut(&LayerId::Session) else {
            return;
        };
        match name {
            key::PARENT => {
                if layer.take_parent(prim).is_some() {
                    self.settle_parent(prim);
                }
            }
            name if is_slot_name(name) => {
                if layer.take_slot(prim, name).is_some() {
                    self.settle_slot(prim, name);
                }
            }
            name => {
                if layer.take_property(prim, name).is_some() {
                    self.settle_property(prim, name);
                }
            }
        }
    }

    pub fn apply_all<'a>(
        &mut self,
        entries: impl IntoIterator<Item = &'a Entry>,
    ) -> Result<(), StateError> {
        for entry in entries {
            self.apply(entry)?;
        }
        Ok(())
    }

    /// The persistent entry set: everything a save would write. Script-created
    /// prims are transient and absent, which is what keeps a spawn/despawn loop
    /// from accumulating in a namespace that never reclaims.
    #[must_use]
    pub fn entries(&self) -> BTreeMap<String, Vec<u8>> {
        let mut out = BTreeMap::new();
        out.insert(
            key::META.to_owned(),
            self.meta.encode().expect("DocMeta always encodes"),
        );

        let Some(document) = self.layers.get(&LayerId::Document) else {
            return out;
        };
        let mut sites = HashSet::new();
        for (prim, opinions) in document.prims() {
            let Some(parent) = opinions.parent().and_then(|(o, _)| o.value()) else {
                continue;
            };
            sites.insert(prim);
            out.insert(key::parent(prim), parent.encode());
            for (name, value) in opinions.set_properties() {
                out.insert(key::prop(prim, name), value.encode());
            }
            for (name, value) in opinions.set_slots() {
                out.insert(key::prop(prim, name), value.to_vec());
            }
        }

        // An override rides on the prim that references the document it speaks
        // for, so one whose site the document layer does not state is absent
        // for the same reason a script-created prim is.
        for (site, overrides) in &self.overrides {
            if !sites.contains(site) {
                continue;
            }
            out.extend(overrides.entries(*site));
        }
        out
    }
}
