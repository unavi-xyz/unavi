//! Writes into a named layer, from entries that arrive in any order.

use crate::{
    attributes::parent::ParentAttr,
    id::PrimId,
    key,
    property::{
        Property,
        name::PropName,
        value::Value,
    },
    state::{
        HsdState,
        StateError,
        entry::{
            Entry,
            Stamp,
        },
        stack::LayerId,
    },
};

impl HsdState {
    /// Writes one entry into `id`. An empty value blocks the key. Keys other
    /// than a prim's properties are ignored.
    pub fn apply(&mut self, id: &LayerId, entry: &Entry) -> Result<(), StateError> {
        let at = self.layer_position(id)?;
        let Some(key::Key::Prop { prim, name }) = key::Key::parse(&entry.key) else {
            return Ok(());
        };
        let stamp = Stamp::new(entry.timestamp, &entry.value);
        let value = (!entry.value.is_empty()).then(|| entry.value.clone());

        if name == ParentAttr::NAME {
            let parent = value
                .as_ref()
                .map(ParentAttr::from_wire)
                .transpose()?
                .flatten();
            self.write_parent(at, prim, parent, Some(stamp));
        } else {
            let property = value.as_ref().map(Value::decode).transpose()?;
            self.write_property(at, prim, &name, property, stamp);
        }
        Ok(())
    }

    /// Drops `id`'s opinion on a key, which is not the same as blocking it.
    pub fn clear(&mut self, id: &LayerId, prim: PrimId, name: &PropName) -> Result<(), StateError> {
        let at = self.layer_position(id)?;
        let layer = &mut self.stack[at].layer;
        if *name == ParentAttr::NAME {
            if layer.take_parent(prim).is_some() {
                self.settle_parent(prim);
            }
        } else if layer.take_property(prim, name).is_some() {
            self.settle_property(prim, name);
        }
        self.forget_if_unstated(prim);
        Ok(())
    }
}
