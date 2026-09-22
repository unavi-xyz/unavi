//! Local writes: applied immediately, needing no capability and no storage.
//! Whether they reach other peers is the space protocol's business, and
//! whether they reach the document is an explicit save.

use crate::{
    attributes::Attribute,
    id::PrimId,
    key,
    property::{
        Parent,
        Property,
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
            Some(parent.map_or(Parent::Root, Parent::Prim)),
            None,
        );
        prim
    }

    pub fn set_parent(&mut self, prim: PrimId, parent: Parent) -> Result<(), StateError> {
        if !self.exists(prim) {
            return Err(StateError::UnknownPrim(prim));
        }
        self.write_parent(LayerId::Runtime, prim, Some(parent), None);
        Ok(())
    }

    /// Blocks the prim in the runtime layer rather than deleting it. A script
    /// can hide a document prim for this session; only a commit can remove one
    /// from the document.
    pub fn remove_prim(&mut self, prim: PrimId) {
        if self.resolved.contains_key(&prim) {
            self.write_parent(LayerId::Runtime, prim, None, None);
        }
    }

    pub fn set_property(
        &mut self,
        prim: PrimId,
        name: &str,
        value: Property,
    ) -> Result<(), StateError> {
        if !key::is_valid_name(name) {
            return Err(StateError::Name(name.to_owned()));
        }
        let stamp = Stamp::now(&value.encode());
        self.write_property(LayerId::Runtime, prim, name, Some(value), stamp);
        Ok(())
    }

    pub fn set_attribute<A: Attribute>(
        &mut self,
        prim: PrimId,
        value: &A,
    ) -> Result<(), StateError> {
        self.set_property(prim, A::KEY, Property::Attribute(value.encode()?))
    }

    pub fn set_relationship(
        &mut self,
        prim: PrimId,
        name: &str,
        target: PrimId,
    ) -> Result<(), StateError> {
        self.set_property(prim, name, Property::Relationship(target))
    }

    pub fn remove_property(&mut self, prim: PrimId, name: &str) {
        let stamp = Stamp::now(&[]);
        self.write_property(LayerId::Runtime, prim, name, None, stamp);
    }

    pub fn set_slot(&mut self, prim: PrimId, name: &str, value: Vec<u8>) -> Result<(), StateError> {
        if !key::is_valid_name(name) {
            return Err(StateError::Name(name.to_owned()));
        }
        let stamp = Stamp::now(&value);
        self.write_slot(LayerId::Runtime, prim, name, Some(value), stamp);
        Ok(())
    }

    pub fn remove_slot(&mut self, prim: PrimId, name: &str) {
        let stamp = Stamp::now(&[]);
        self.write_slot(LayerId::Runtime, prim, name, None, stamp);
    }
}
