use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use bevy::prelude::*;
use bevy_hsd::HsdRecordId;
use blake3::Hash;

#[derive(Clone, Default)]
pub struct TransformHandles {
    pub local: Arc<RwLock<Transform>>,
    pub global: Arc<RwLock<GlobalTransform>>,
}

#[derive(Component)]
pub struct OutboundTransform(pub TransformHandles);

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
