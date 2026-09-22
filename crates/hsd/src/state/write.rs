//! Local writes: applied immediately, needing no capability and no storage.
//! Whether they reach other peers is the space protocol's business, and
//! whether they reach the document is an explicit save.

use crate::{
    attributes::{
        Attribute,
        parent::ParentAttr,
    },
    id::PrimId,
    key,
    property::Property,
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
        // `parent` is structural, not a property: it decides realization, and
        // writing it here would put a second value under the name that
        // [`Self::set_parent`] already owns.
        if !key::is_valid_name(name) || name == ParentAttr::KEY {
            return Err(StateError::Name(name.to_owned()));
        }
        let stamp = Stamp::for_property(&value);
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
        if name == ParentAttr::KEY {
            return;
        }
        let stamp = Stamp::now(&[]);
        self.write_property(LayerId::Runtime, prim, name, None, stamp);
    }
}
