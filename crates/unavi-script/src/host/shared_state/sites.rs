//! Which reference site places each referenced document, for scripts to read
//! off-thread.

use std::sync::Arc;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_hsd::{
    document::HsdDocId,
    prim::{
        Prim,
        PrimOf,
    },
    reference::ReferenceInstance,
};
use hsd::id::{
    DocId,
    PrimId,
};
use parking_lot::RwLock;

/// Each placed instance's site: the referencing document and prim.
#[derive(Resource, Clone, Default)]
pub struct Sites(Arc<RwLock<HashMap<DocId, (DocId, PrimId)>>>);

impl Sites {
    #[must_use]
    pub fn get(&self, doc: &DocId) -> Option<(DocId, PrimId)> {
        self.0.read().get(doc).copied()
    }
}

pub fn register_site(
    trigger: On<Insert, ReferenceInstance>,
    instances: Query<(&HsdDocId, &ReferenceInstance)>,
    prims: Query<(&Prim, &PrimOf)>,
    docs: Query<&HsdDocId>,
    sites: Res<Sites>,
) {
    let Ok((instance, reference)) = instances.get(trigger.entity) else {
        return;
    };
    let Ok((prim, owner)) = prims.get(reference.prim) else {
        return;
    };
    let Ok(host) = docs.get(owner.0) else {
        return;
    };
    sites.0.write().insert(instance.0, (host.0, prim.0));
}

pub fn deregister_site(
    trigger: On<Remove, ReferenceInstance>,
    instances: Query<&HsdDocId>,
    sites: Res<Sites>,
) {
    if let Ok(instance) = instances.get(trigger.entity) {
        sites.0.write().remove(&instance.0);
    }
}
