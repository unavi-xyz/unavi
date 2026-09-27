//! The projected layers. Each entry is a key's winner as the store chose it,
//! and replaces whatever the layer held for that key. An empty value is a key
//! the store holds no value for.

use crate::{
    format::meta::{
        DOC_VERSION,
        DocMeta,
    },
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
        opinion::Opinion,
    },
};

impl HsdState {
    /// Projects one of this document's entries. A value that fails to decode
    /// still clears what the key held, since it is no longer the store's
    /// winner.
    pub fn project(&mut self, entry: &Entry) -> Result<(), StateError> {
        let stamp = Stamp::new(entry.timestamp, &entry.value);
        let value = (!entry.value.is_empty()).then_some(entry.value.as_slice());

        match key::Key::parse(&entry.key) {
            Some(key::Key::Meta) => self.project_meta(value),
            Some(key::Key::Prop { prim, name }) if name == ParentAttr::NAME => {
                match value.map(ParentAttr::from_wire).transpose() {
                    Ok(parent) => {
                        self.project_parent(prim, parent.flatten(), stamp);
                        Ok(())
                    }
                    Err(err) => {
                        self.project_parent(prim, None, stamp);
                        Err(err.into())
                    }
                }
            }
            Some(key::Key::Prop { prim, name }) => match value.map(Value::decode).transpose() {
                Ok(property) => {
                    self.project_property(prim, &name, property, stamp);
                    Ok(())
                }
                Err(err) => {
                    self.project_property(prim, &name, None, stamp);
                    Err(err.into())
                }
            },
            Some(key::Key::Nested { prim, group, tail }) if group == reference_attr::GROUP => {
                LayerKey::parse(&tail).map_or(Ok(()), |layer_key| {
                    self.write_reference(prim, &layer_key, value, stamp)
                })
            }
            Some(key::Key::Nested { .. }) | None => Ok(()),
        }
    }

    /// Projects an entry of the referencing document into the override layer.
    /// Keys outside that document's reference layers are ignored.
    pub fn project_override(&mut self, entry: &Entry) -> Result<(), StateError> {
        let Some((_, LayerKey { target, name })) = LayerKey::parse_key(&entry.key) else {
            return Ok(());
        };
        let stamp = Stamp::new(entry.timestamp, &entry.value);
        let value = (!entry.value.is_empty()).then_some(entry.value.as_slice());

        if name == ParentAttr::NAME {
            let parent = value.map(ParentAttr::from_wire).transpose()?.flatten();
            self.write_parent(LayerId::Override, target, parent, Some(stamp));
        } else {
            let property = value.map(Value::decode).transpose()?;
            self.write_property(LayerId::Override, target, &name, property, stamp);
        }
        Ok(())
    }

    /// Whether the document states a format newer than this build reads.
    /// A refused document realizes nothing.
    #[must_use]
    pub const fn is_refused(&self) -> bool {
        self.meta.version > DOC_VERSION
    }

    fn project_meta(&mut self, value: Option<&[u8]>) -> Result<(), StateError> {
        let meta = value.map(DocMeta::decode).transpose()?.unwrap_or_default();
        let was_refused = self.is_refused();
        self.meta = meta;
        if was_refused == self.is_refused() {
            return Ok(());
        }

        let prims = if was_refused {
            self.resolved.keys().copied().collect::<Vec<_>>()
        } else {
            self.roots()
        };
        for prim in prims {
            self.refresh(prim);
        }
        Ok(())
    }

    fn project_parent(&mut self, prim: PrimId, parent: Option<ParentAttr>, stamp: Stamp) {
        let layer = self.layer_mut(LayerId::Document);
        match parent {
            Some(parent) => layer
                .entry(prim)
                .replace_parent(Opinion::Set(parent), stamp),
            None => {
                layer.take_parent(prim);
            }
        }
        self.settle_parent(prim);
        self.forget_if_unstated(prim);
    }

    fn project_property(
        &mut self,
        prim: PrimId,
        name: &PropName,
        value: Option<Value>,
        stamp: Stamp,
    ) {
        let layer = self.layer_mut(LayerId::Document);
        match value {
            Some(value) => layer
                .entry(prim)
                .replace_property(name, Opinion::Set(value), stamp),
            None => {
                layer.take_property(prim, name);
            }
        }
        self.settle_property(prim, name);
        self.forget_if_unstated(prim);
    }
}
