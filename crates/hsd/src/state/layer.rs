use std::collections::HashMap;

use crate::{
    id::PrimId,
    state::opinion::PrimOpinions,
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

    pub(super) fn entry(&mut self, prim: PrimId) -> &mut PrimOpinions {
        self.0.entry(prim).or_insert_with(PrimOpinions::new)
    }

    pub(super) fn prims(&self) -> impl Iterator<Item = (PrimId, &PrimOpinions)> {
        self.0.iter().map(|(prim, opinions)| (*prim, opinions))
    }
}
