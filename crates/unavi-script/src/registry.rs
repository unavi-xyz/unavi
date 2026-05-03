use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use bevy::prelude::*;
use bevy_hsd::HsdRecordId;
use blake3::Hash;

/// Mirrors an entity's `Transform` / `GlobalTransform` into Arc-backed handles
/// each frame so async script code can read without queuing an ECS roundtrip.
#[derive(Clone)]
pub struct TransformHandles {
    pub local: Arc<RwLock<Transform>>,
    pub global: Arc<RwLock<GlobalTransform>>,
}

impl Default for TransformHandles {
    fn default() -> Self {
        Self {
            local: Arc::new(RwLock::new(Transform::default())),
            global: Arc::new(RwLock::new(GlobalTransform::default())),
        }
    }
}

/// Attached to every `HsdRecordId` entity so the sync system can write into it.
#[derive(Component)]
pub struct OutboundTransform(pub TransformHandles);

/// Maps document hash → [`TransformHandles`].
/// Shared between Bevy (as a Resource) and `WiredSceneBackend` (as a cloned Arc).
#[derive(Resource, Clone)]
pub struct DocTransformRegistry(pub Arc<Mutex<HashMap<Hash, TransformHandles>>>);

pub fn on_hsd_record_added(
    trigger: On<Add, HsdRecordId>,
    query: Query<&HsdRecordId>,
    registry: Res<DocTransformRegistry>,
    mut commands: Commands,
) {
    let entity = trigger.event_target();
    let Ok(record_id) = query.get(entity) else {
        return;
    };
    let handles = TransformHandles::default();
    registry
        .0
        .lock()
        .expect("registry poisoned")
        .insert(record_id.0, handles.clone());
    commands.entity(entity).insert(OutboundTransform(handles));
}

pub fn sync_outbound_transforms(query: Query<(&OutboundTransform, &Transform, &GlobalTransform)>) {
    for (outbound, transform, global) in &query {
        *outbound.0.local.write().expect("local transform poisoned") = *transform;
        *outbound
            .0
            .global
            .write()
            .expect("global transform poisoned") = *global;
    }
}
