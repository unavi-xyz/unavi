//! The session layer. Opinions arrive from peers in any order, so a write
//! older than the one held is refused.

use crate::{
    id::PrimId,
    key,
    property::{
        Property,
        name::PropName,
        value::Value,
    },
    schema::parent::ParentAttr,
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
    /// An empty value blocks the key. Keys other than a prim's properties are
    /// ignored.
    pub fn apply_session(&mut self, entry: &Entry) -> Result<(), StateError> {
        let Some(key::Key::Prop { prim, name }) = key::Key::parse(&entry.key) else {
            return Ok(());
        };
        let stamp = Stamp::new(entry.timestamp, &entry.value);
        let value = (!entry.value.is_empty()).then_some(entry.value.as_slice());

        if name == ParentAttr::NAME {
            let parent = value.map(ParentAttr::from_wire).transpose()?.flatten();
            self.write_parent(LayerId::Session, prim, parent, Some(stamp));
        } else {
            let property = value.map(Value::decode).transpose()?;
            self.write_property(LayerId::Session, prim, &name, property, stamp);
        }
        Ok(())
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
}
