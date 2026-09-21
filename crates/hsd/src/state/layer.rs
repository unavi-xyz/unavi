use std::collections::HashMap;

use crate::{
    id::PrimId,
    state::opinion::{
        Origin,
        PrimOpinions,
    },
};

/// Strength order: a later variant's opinion wins over an earlier one's.
///
/// `Document` is what a keyholder wrote and what a save writes back.
/// `Runtime` is what this peer's scripts said this session, and is replicated
/// by nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum LayerId {
    Document,
    Runtime,
}

impl LayerId {
    pub(super) const ALL: [Self; 2] = [Self::Document, Self::Runtime];
}

/// One layer's opinions, keyed by prim.
#[derive(Debug, Default)]
pub(super) struct Layer(HashMap<PrimId, PrimOpinions>);

impl Layer {
    pub(super) fn get(&self, prim: PrimId) -> Option<&PrimOpinions> {
        self.0.get(&prim)
    }

    pub(super) fn entry(&mut self, prim: PrimId, origin: Origin) -> &mut PrimOpinions {
        self.0
            .entry(prim)
            .or_insert_with(|| PrimOpinions::new(origin))
    }

    pub(super) fn contains(&self, prim: PrimId) -> bool {
        self.0.contains_key(&prim)
    }

    /// Drops everything this layer says about `prim`. A weaker layer's
    /// opinions are untouched, so the prim may still resolve.
    pub(super) fn remove(&mut self, prim: PrimId) {
        self.0.remove(&prim);
    }

    pub(super) fn prims(&self) -> impl Iterator<Item = (PrimId, &PrimOpinions)> {
        self.0.iter().map(|(prim, opinions)| (*prim, opinions))
    }
}
