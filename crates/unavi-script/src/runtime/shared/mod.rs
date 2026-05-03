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
