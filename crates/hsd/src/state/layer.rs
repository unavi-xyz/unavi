use std::collections::HashMap;

use crate::{
    id::PrimId,
    property::{
        Parent,
        Property,
    },
    state::{
        entry::Stamp,
        opinion::{
            Opinion,
            PrimOpinions,
        },
    },
};

/// Strength order: a later variant's opinion wins over an earlier one's.
///
/// `Document` is what a keyholder wrote and what a save writes back.
/// `Runtime` is what this peer's scripts said this session, and is replicated
/// by nothing. `Session` is what a commit falls back to when the caller holds
/// no durable key; it resolves above `Runtime` and, like it, is replicated by
/// nothing yet — replication of session opinions is the session-layer
/// workstream's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum LayerId {
    Document,
    Runtime,
    Session,
}

impl LayerId {
    pub(super) const ALL: [Self; 3] = [Self::Document, Self::Runtime, Self::Session];
}

/// One layer's opinions, keyed by prim.
#[derive(Debug, Default)]
pub(super) struct Layer(HashMap<PrimId, PrimOpinions>);

impl Layer {
    pub(super) fn get(&self, prim: PrimId) -> Option<&PrimOpinions> {
        self.0.get(&prim)
    }

    pub(super) fn entry(&mut self, prim: PrimId) -> &mut PrimOpinions {
        self.0.entry(prim).or_insert_with(PrimOpinions::new)
    }

    pub(super) fn prims(&self) -> impl Iterator<Item = (PrimId, &PrimOpinions)> {
        self.0.iter().map(|(prim, opinions)| (*prim, opinions))
    }

    /// Removes this layer's opinion on a key, answering what it was. Used by
    /// `commit`, which lifts a live opinion into its target and must then drop
    /// it from every live layer so the key resolves from the target alone.
    pub(super) fn take_parent(&mut self, prim: PrimId) -> Option<(Opinion<Parent>, Stamp)> {
        let opinions = self.0.get_mut(&prim)?;
        let taken = opinions.take_parent();
        if opinions.is_empty() {
            self.0.remove(&prim);
        }
        taken
    }

    pub(super) fn take_property(
        &mut self,
        prim: PrimId,
        name: &str,
    ) -> Option<(Opinion<Property>, Stamp)> {
        let opinions = self.0.get_mut(&prim)?;
        let taken = opinions.take_property(name);
        if opinions.is_empty() {
            self.0.remove(&prim);
        }
        taken
    }

    pub(super) fn take_slot(
        &mut self,
        prim: PrimId,
        name: &str,
    ) -> Option<(Opinion<Vec<u8>>, Stamp)> {
        let opinions = self.0.get_mut(&prim)?;
        let taken = opinions.take_slot(name);
        if opinions.is_empty() {
            self.0.remove(&prim);
        }
        taken
    }
}
