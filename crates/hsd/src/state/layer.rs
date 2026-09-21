use std::collections::{
    BTreeSet,
    HashMap,
};

use crate::{
    id::PrimId,
    state::prim::PrimState,
};

/// One layer's opinions about prims, shaped like the entry set so that saving
/// a layer is a per-key diff rather than a snapshot.
#[derive(Debug, Default)]
pub(super) struct Layer {
    pub(super) prims:    HashMap<PrimId, PrimState>,
    /// Parent id to children, including parents that do not exist yet, which
    /// is what lets an orphan be picked up when its parent arrives.
    pub(super) children: HashMap<PrimId, BTreeSet<PrimId>>,
}
