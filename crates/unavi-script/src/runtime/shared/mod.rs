use std::sync::Arc;

use tokio::sync::Mutex as TokioMutex;

use crate::runtime::shared::wired::{input::WiredInputBackend, scene::WiredSceneBackend};

mod slot_map;
pub mod wired;

#[derive(Clone)]
pub struct RuntimeBackend {
    pub wired_input: Arc<TokioMutex<WiredInputBackend>>,
    pub wired_scene: Arc<TokioMutex<WiredSceneBackend>>,
}
