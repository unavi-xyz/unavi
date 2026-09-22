//! Applying entries, from the document or from a compiled package. Entries
//! arrive unordered — a child may be seen before its parent — so every path
//! here has to be order-independent.

use std::collections::BTreeMap;

use crate::{
    attributes::{
        Attribute,
        parent::ParentAttr,
        reference::{
            self,
            RefKey,
            ReferenceAttr,
        },
    },
    id::PrimId,
    key,
    meta::DocMeta,
    property::Property,
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
            Some(key::Key::Prop { prim, name }) if name == ParentAttr::KEY => {
                let parent = if empty {
                    None
                } else {
                    ParentAttr::from_wire(&entry.value)?
                };
                self.write_parent(layer, prim, parent, Some(stamp));
            }
            // `ref` owns the namespace below it, so its own key is a spine:
            // data there would prefix-delete the target and the layer both.
            Some(key::Key::Prop { name, .. }) if name == ReferenceAttr::KEY => {}
            Some(key::Key::Prop { prim, name }) => {
                // The tag says whether the value is a property or a blob, so
                // no name list classifies it and a value this build has never
                // heard of travels as bytes.
                let value = if empty {
                    None
                } else {
                    Some(Property::decode(&entry.value)?)
                };
                self.write_property(layer, prim, &name, value, stamp);
            }
            Some(key::Key::PropSub { prim, name, tail }) if name == ReferenceAttr::KEY => {
                match reference::parse_tail(&tail) {
                    // The reference target is the ordinary `ref` property: it
                    // settles and emits like any other, and
                    // `p/<site>/ref/target/` is only its wire shape.
                    Some(RefKey::Target) => {
                        let value = if empty {
                            None
                        } else {
                            Some(Property::decode(&entry.value)?)
                        };
                        self.write_property(layer, prim, ReferenceAttr::KEY, value, stamp);
                    }
                    // A reference-layer opinion is durable in the document
                    // stating it, so it arrives by sync and never as a session
                    // opinion. It does not compose here; the realizer installs
                    // it into the referenced document.
                    Some(RefKey::Layer { target, name }) if layer == LayerId::Document => {
                        self.write_reference(
                            prim,
                            target,
                            &name,
                            (!empty).then_some(&entry.value),
                            stamp,
                        )?;
                    }
                    Some(RefKey::Layer { .. }) | None => {}
                }
            }
            Some(key::Key::PropSub { .. }) | None => {}
        }
        Ok(())
    }

    /// Drops the session opinion on a key, so whatever the layers beneath it
    /// say composes again.
    ///
    /// Not the same as blocking the key: a block is an opinion, and this is
    /// the absence of one. What a peer leaving takes with it.
    pub fn clear_session(&mut self, prim: PrimId, name: &str) {
        match name {
            ParentAttr::KEY => {
                if self.layers[LayerId::Session.idx()]
                    .take_parent(prim)
                    .is_some()
                {
                    self.settle_parent(prim);
                }
            }
            name => {
                if self.layers[LayerId::Session.idx()]
                    .take_property(prim, name)
                    .is_some()
                {
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

        let Some(document) = self.layers.get(LayerId::Document.idx()) else {
            return out;
        };
        for (prim, opinions) in document.prims() {
            let Some(parent) = opinions.parent().and_then(|(o, _)| o.value()) else {
                continue;
            };
            out.insert(
                key::prop(prim, ParentAttr::KEY),
                ParentAttr::to_wire(Some(*parent)),
            );
            for (name, value) in opinions.set_properties() {
                // `ref` owns a namespace rather than its own key, so its value
                // goes to the slot the attribute names.
                let key = if name == ReferenceAttr::KEY {
                    reference::target_key(prim)
                } else {
                    key::prop(prim, name)
                };
                out.insert(key, value.encode());
            }
        }

        // A reference layer rides on the prim that references the document it
        // speaks for, so one whose site the document layer does not state is
        // absent for the same reason a script-created prim is.
        for (site, layer) in &self.references {
            let stated = document
                .get(*site)
                .and_then(|opinions| opinions.parent())
                .is_some_and(|(opinion, _)| opinion.value().is_some());
            if !stated {
                continue;
            }
            for (target, opinions) in layer.prims() {
                if let Some((opinion, _)) = opinions.parent() {
                    out.insert(
                        reference::layer_key(*site, target, ParentAttr::KEY),
                        ParentAttr::to_wire(opinion.value().copied()),
                    );
                }
                for (name, opinion) in opinions.properties() {
                    out.insert(
                        reference::layer_key(*site, target, name),
                        opinion.value().map(Property::encode).unwrap_or_default(),
                    );
                }
            }
        }
        out
    }
}
