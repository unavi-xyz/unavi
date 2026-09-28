//! Local writes, into the runtime layer.

use crate::{
    attributes::parent::ParentAttr,
    id::PrimId,
    property::{
        Payload,
        Property,
        name::PropName,
        value::Value,
    },
    state::{
        HsdState,
        StateError,
        entry::Stamp,
        layer::LayerId,
    },
};

impl HsdState {
    pub fn create_prim(&mut self, parent: Option<PrimId>) -> PrimId {
        let prim = PrimId::new();
        self.write_parent(
            LayerId::Runtime,
            prim,
            Some(parent.map_or(ParentAttr::Root, ParentAttr::Prim)),
            None,
        );
        prim
    }

    pub fn set_parent(&mut self, prim: PrimId, parent: ParentAttr) -> Result<(), StateError> {
        if !self.exists(prim) {
            return Err(StateError::UnknownPrim(prim));
        }
        self.write_parent(LayerId::Runtime, prim, Some(parent), None);
        Ok(())
    }

    /// A prim only the runtime layer states is forgotten. Any other is
    /// blocked in the runtime layer, and stays in the layers beneath it.
    pub fn remove_prim(&mut self, prim: PrimId) {
        if !self.resolved.contains_key(&prim) {
            return;
        }
        let runtime_only = LayerId::ALL
            .into_iter()
            .filter(|id| *id != LayerId::Runtime)
            .all(|id| self.layer(id).get(prim).is_none());
        if !runtime_only {
            self.write_parent(LayerId::Runtime, prim, None, None);
            return;
        }

        self.layer_mut(LayerId::Runtime).remove(prim);
        self.settle_parent(prim);
        self.forget_if_unstated(prim);
    }

    pub fn set_property(
        &mut self,
        prim: PrimId,
        name: &PropName,
        value: Value,
    ) -> Result<(), StateError> {
        if *name == ParentAttr::NAME {
            return Err(StateError::Reserved(name.clone()));
        }
        let timestamp = self.local_property_time(LayerId::Runtime, prim, name);
        let stamp = Stamp::of_property(timestamp, &value);
        self.write_property(LayerId::Runtime, prim, name, Some(value), stamp);
        Ok(())
    }

    pub fn set_attribute<A: Property>(
        &mut self,
        prim: PrimId,
        value: &A,
    ) -> Result<(), StateError> {
        self.set_payload(prim, &A::NAME, value)
    }

    pub fn set_payload<V: Payload>(
        &mut self,
        prim: PrimId,
        name: &PropName,
        value: &V,
    ) -> Result<(), StateError> {
        self.set_property(prim, name, Value::Attribute(value.encode()?.into()))
    }

    pub fn set_relationship(
        &mut self,
        prim: PrimId,
        name: &PropName,
        target: PrimId,
    ) -> Result<(), StateError> {
        self.set_property(prim, name, Value::Relationship(target))
    }

    pub fn remove_property(&mut self, prim: PrimId, name: &PropName) {
        if *name == ParentAttr::NAME {
            return;
        }
        let timestamp = self.local_property_time(LayerId::Runtime, prim, name);
        let stamp = Stamp::new(timestamp, &[]);
        self.write_property(LayerId::Runtime, prim, name, None, stamp);
    }

    /// Removes every property of `group` the prim resolves.
    pub fn remove_group(&mut self, prim: PrimId, group: &str) {
        let Some(state) = self.resolved.get(&prim) else {
            return;
        };
        let names = state
            .properties()
            .filter(|(name, _)| name.group() == group)
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        for name in names {
            self.remove_property(prim, &name);
        }
    }
}
