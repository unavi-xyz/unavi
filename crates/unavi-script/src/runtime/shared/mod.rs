use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use blake3::Hash;
use tokio::sync::Mutex as TokioMutex;
use wired::scene::SceneContext;

use crate::{
    registry::TransformHandles,
    runtime::shared::wired::{input::WiredInputBackend, scene::WiredSceneBackend},
};

mod slot_map;
pub mod wired;

#[derive(Clone)]
pub struct RuntimeBackend {
    pub wired_input: Arc<TokioMutex<WiredInputBackend>>,
    pub wired_scene: Arc<TokioMutex<WiredSceneBackend>>,
}

impl RuntimeBackend {
    pub fn new(
        ctx: SceneContext,
        transform_registry: Arc<Mutex<HashMap<Hash, TransformHandles>>>,
    ) -> Self {
        Self {
            wired_input: Arc::default(),
            wired_scene: Arc::new(TokioMutex::new(WiredSceneBackend::new(
                ctx,
                transform_registry,
            ))),
        }
    }
}
