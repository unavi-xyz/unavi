use std::sync::LazyLock;

use bevy::math::Vec3;
use parking_lot::RwLock;

use crate::runtime::shared::registry::{
    event::{
        EVENT_RECEPTOR_REGISTRY,
        ReceptorScope,
    },
    transform::NODE_TRANSFORM_REGISTRY,
};

pub struct SpatialReceptor {
    pub channels: Vec<String>,
    pub position: Vec3,
    pub radius:   f32,
}

pub fn spatial_receptors() -> Vec<SpatialReceptor> {
    let transforms = NODE_TRANSFORM_REGISTRY.read();
    EVENT_RECEPTOR_REGISTRY
        .read()
        .values()
        .filter_map(|entry| match &entry.scope {
            ReceptorScope::Spatial { node, radius } => {
                transforms.get(node).map(|t| SpatialReceptor {
                    channels: entry.channels.clone(),
                    position: t.global.translation(),
                    radius:   *radius,
                })
            }
            ReceptorScope::Global => None,
        })
        .collect()
}

pub type EmitObserver = Box<dyn Fn(&str, Vec3, f32) + Send + Sync>;

pub static EMIT_OBSERVER: LazyLock<RwLock<Option<EmitObserver>>> =
    LazyLock::new(|| RwLock::new(None));

pub(crate) fn record_emit(channel: &str, position: Vec3, radius: f32) {
    if let Some(observer) = EMIT_OBSERVER.read().as_ref() {
        observer(channel, position, radius);
    }
}
