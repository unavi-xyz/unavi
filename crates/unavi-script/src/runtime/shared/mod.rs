use std::sync::Arc;

use tokio::sync::Mutex;
use wired::scene::SceneContext;

mod slot_map;
pub mod wired;

#[derive(Clone)]
pub struct RuntimeBackend {
    pub wired_scene: Arc<Mutex<wired::scene::WiredSceneBackend>>,
}

impl RuntimeBackend {
    pub fn new(ctx: SceneContext) -> Self {
        Self {
            wired_scene: Arc::new(Mutex::new(wired::scene::WiredSceneBackend::new(ctx))),
        }
    }
}
