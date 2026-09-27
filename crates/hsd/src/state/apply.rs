//! Entries arrive in any order, so every path here is order-independent.

use std::collections::BTreeMap;

use crate::{
    format::meta::DocMeta,
    id::PrimId,
    key,
    property::{
        Property,
        name::PropName,
        value::Value,
    },
    schema::{
        parent::ParentAttr,
        reference::{
            self as reference_attr,
            LayerKey,
        },
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

    pub fn apply_all<'a>(
        &mut self,
        entries: impl IntoIterator<Item = &'a Entry>,
    ) -> Result<(), StateError> {
        for entry in entries {
            self.apply(entry)?;
        }
        Ok(())
    }

    /// Applies an entry to the session layer. An empty value blocks the key.
    pub fn apply_session(&mut self, entry: &Entry) -> Result<(), StateError> {
        self.apply_at(LayerId::Session, entry)
    }

    /// Drops the session opinion on a key, which is not the same as blocking
    /// it.
    pub fn clear_session(&mut self, prim: PrimId, name: &PropName) {
        let session = self.layer_mut(LayerId::Session);
        if *name == ParentAttr::NAME {
            if session.take_parent(prim).is_some() {
                self.settle_parent(prim);
            }
        } else if session.take_property(prim, name).is_some() {
            self.settle_property(prim, name);
        }
    }

    fn apply_at(&mut self, layer: LayerId, entry: &Entry) -> Result<(), StateError> {
        let stamp = Stamp::new(entry.timestamp, &entry.value);
        let value = (!entry.value.is_empty()).then_some(entry.value.as_slice());

        match key::Key::parse(&entry.key) {
            Some(key::Key::Meta) => {
                if let Some(value) = value
                    && layer == LayerId::Document
                {
                    self.meta = DocMeta::decode(value)?;
                }
            }
            Some(key::Key::Prop { prim, name }) if name == ParentAttr::NAME => {
                let parent = match value {
                    Some(value) => ParentAttr::from_wire(value)?,
                    None => None,
                };
                self.write_parent(layer, prim, parent, Some(stamp));
            }
            Some(key::Key::Prop { prim, name }) => {
                let value = value.map(Value::decode).transpose()?;
                self.write_property(layer, prim, &name, value, stamp);
            }
            Some(key::Key::Nested { prim, group, tail })
                if group == reference_attr::GROUP && layer == LayerId::Document =>
            {
                if let Some(layer_key) = LayerKey::parse(&tail) {
                    self.write_reference(prim, &layer_key, value, stamp)?;
                }
            }
            Some(key::Key::Nested { .. }) | None => {}
        }
        Ok(())
    }

    /// Everything a save writes. Prims the document layer does not state are
    /// absent, and so are the reference layers they carry.
    #[must_use]
    pub fn entries(&self) -> BTreeMap<String, Vec<u8>> {
        let mut out = BTreeMap::new();
        out.insert(
            key::META.to_owned(),
            self.meta.encode().expect("DocMeta always encodes"),
        );

        let document = self.layer(LayerId::Document);
        for (prim, opinions) in document.prims() {
            let Some(parent) = opinions.parent().and_then(|(o, _)| o.value()) else {
                continue;
            };
            out.insert(
                key::Key::prop(prim, &ParentAttr::NAME).to_string(),
                ParentAttr::to_wire(Some(*parent)),
            );
            for (name, value) in opinions.set_properties() {
                out.insert(key::Key::prop(prim, name).to_string(), value.encode());
            }
        }

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
                    let key = LayerKey {
                        target,
                        name: ParentAttr::NAME,
                    };
                    out.insert(
                        key.key(*site),
                        ParentAttr::to_wire(opinion.value().copied()),
                    );
                }
                for (name, opinion) in opinions.properties() {
                    let key = LayerKey {
                        target,
                        name: name.clone(),
                    };
                    out.insert(
                        key.key(*site),
                        opinion.value().map(Value::encode).unwrap_or_default(),
                    );
                }
            }
        }
        out
    }
}
