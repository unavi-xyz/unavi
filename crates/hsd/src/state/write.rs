//! Plain writes, into the edit target.

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
    },
};

impl HsdState {
    pub fn create_prim(&mut self, parent: Option<PrimId>) -> PrimId {
        let prim = PrimId::new();
        self.write_parent(
            self.edit_position(),
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
        self.write_parent(self.edit_position(), prim, Some(parent), None);
        Ok(())
    }

    /// A prim only the edit target states is forgotten. Any other is blocked
    /// in the edit target, and stays in the layers that state it.
    pub fn remove_prim(&mut self, prim: PrimId) {
        if !self.resolved.contains_key(&prim) {
            return;
        }
        let at = self.edit_position();
        let target_only = self
            .stack
            .iter()
            .enumerate()
            .all(|(i, stacked)| i == at || stacked.layer.get(prim).is_none());
        if !target_only {
            self.write_parent(at, prim, None, None);
            return;
        }

        self.stack[at].layer.remove(prim);
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
        let at = self.edit_position();
        let timestamp = self.local_property_time(at, prim, name);
        let stamp = Stamp::of_property(timestamp, &value);
        self.write_property(at, prim, name, Some(value), stamp);
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
        let at = self.edit_position();
        let timestamp = self.local_property_time(at, prim, name);
        let stamp = Stamp::new(timestamp, &[]);
        self.write_property(at, prim, name, None, stamp);
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
