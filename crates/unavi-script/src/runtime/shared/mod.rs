use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use blake3::Hash;
use tokio::sync::Mutex as TokioMutex;
use wired::scene::SceneContext;

use crate::registry::TransformHandles;

mod slot_map;
pub mod wired;

#[derive(Clone)]
pub struct RuntimeBackend {
    pub wired_scene: Arc<TokioMutex<wired::scene::WiredSceneBackend>>,
}

impl RuntimeBackend {
    pub fn new(
        ctx: SceneContext,
        transform_registry: Arc<Mutex<HashMap<Hash, TransformHandles>>>,
    ) -> Self {
        Self {
            wired_scene: Arc::new(TokioMutex::new(wired::scene::WiredSceneBackend::new(
                ctx,
                transform_registry,
            ))),
        }
    }
}
