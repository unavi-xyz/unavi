use std::sync::{
    Arc,
    Mutex,
    MutexGuard,
};

use bevy::{
    ecs::{
        lifecycle::HookContext,
        world::DeferredWorld,
    },
    platform::collections::HashMap,
    prelude::*,
};
use hsd::{
    id::DocId,
    state::HsdState,
};
use iroh::EndpointAddr;
use unavi_store::Document;

use crate::{
    prim::{
        PrimIndex,
        Prims,
    },
    stack,
};

/// An HSD document, placed or not.
#[derive(Component, Clone)]
#[require(Prims, PrimIndex, Transform, Visibility)]
pub struct Hsd(Arc<Mutex<HsdState>>);

impl Hsd {
    /// Wraps `state` with [`stack::install`]'s layers above its base.
    #[must_use]
    pub fn new(state: HsdState) -> Self {
        Self::shared(Arc::new(Mutex::new(state)))
    }

    /// Wraps a state other holders keep a handle to, installing
    /// [`stack::install`]'s layers in it.
    #[must_use]
    pub fn shared(state: Arc<Mutex<HsdState>>) -> Self {
        let hsd = Self(state);
        if let Some(mut state) = hsd.lock()
            && let Err(err) = stack::install(&mut state)
        {
            warn!(?err, "document stack refused the engine's layers");
        }
        hsd
    }

    /// The shared state this component wraps.
    #[must_use]
    pub fn handle(&self) -> Arc<Mutex<HsdState>> {
        Arc::clone(&self.0)
    }

    /// The document's metadata, cloned out of the state lock.
    #[must_use]
    pub fn meta(&self) -> Option<hsd::format::meta::DocMeta> {
        self.lock().map(|state| state.meta().clone())
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
#[component(on_insert = index_document, on_discard = unindex_document)]
pub struct HsdDocId(pub DocId);

/// Every entity carrying an [`HsdDocId`], by document id, so a lookup never
/// scans the world. Kept by [`HsdDocId`]'s hooks; a world without this
/// resource keeps none.
#[derive(Resource, Default)]
pub struct DocIndex(HashMap<DocId, Entity>);

impl DocIndex {
    #[must_use]
    pub fn get(&self, doc: DocId) -> Option<Entity> {
        self.0.get(&doc).copied()
    }
}

fn index_document(mut world: DeferredWorld, ctx: HookContext) {
    let Some(id) = world.get::<HsdDocId>(ctx.entity).map(|id| id.0) else {
        return;
    };
    if let Some(mut index) = world.get_resource_mut::<DocIndex>() {
        index.0.insert(id, ctx.entity);
    }
}

/// Leaves the id alone if another entity has since claimed it.
fn unindex_document(mut world: DeferredWorld, ctx: HookContext) {
    let Some(id) = world.get::<HsdDocId>(ctx.entity).map(|id| id.0) else {
        return;
    };
    if let Some(mut index) = world.get_resource_mut::<DocIndex>()
        && index.0.get(&id) == Some(&ctx.entity)
    {
        index.0.remove(&id);
    }
}

/// Keeps the namespace open, which protects it from retention.
#[derive(Component, Debug, Clone)]
pub struct HsdNamespace(pub Document);

/// A namespace-backed document this node minted, and so authors. Set only by
/// the code that created the namespace, alongside its [`HsdDocId`].
#[derive(Component, Debug, Clone, Copy)]
pub struct Minted;

/// Endpoints a document syncs from, inherited by every reference it opens.
/// Absent on a locally created document.
#[derive(Component, Debug, Clone, Default)]
pub struct SyncPeers(pub Vec<EndpointAddr>);
