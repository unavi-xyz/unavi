use std::sync::{
    Arc,
    Mutex,
    MutexGuard,
};

use bevy::prelude::*;
use hsd::{
    id::DocId,
    state::HsdState,
};
use iroh::EndpointAddr;
use unavi_store::Document;

use crate::prim::{
    PrimIndex,
    Prims,
};

/// An HSD document, placed or not.
#[derive(Component, Clone)]
#[require(Prims, PrimIndex, Transform, Visibility)]
pub struct Hsd(pub Arc<Mutex<HsdState>>);

impl Hsd {
    #[must_use]
    pub fn new(state: HsdState) -> Self {
        Self(Arc::new(Mutex::new(state)))
    }

    /// `None` on a poisoned lock.
    #[must_use]
    pub fn lock(&self) -> Option<MutexGuard<'_, HsdState>> {
        self.0
            .lock()
            .inspect_err(|_| warn!("document state poisoned"))
            .ok()
    }
}

/// A document that is not in the scene. Its scene events are discarded until
/// [`crate::anchor::place`] removes this.
#[derive(Component)]
pub struct Unplaced;

/// The namespace id of a namespace-backed document, or the site id of a
/// reference instance.
#[derive(Component, Debug, Clone, Copy)]
pub struct HsdDocId(pub DocId);

/// Keeps the namespace open, which protects it from retention.
#[derive(Component, Debug, Clone)]
pub struct HsdNamespace(pub Document);

/// Endpoints a document syncs from, inherited by every reference it opens.
/// Absent on a locally created document.
#[derive(Component, Debug, Clone, Default)]
pub struct SyncPeers(pub Vec<EndpointAddr>);
