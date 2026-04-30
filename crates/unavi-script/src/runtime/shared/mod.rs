#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

pub mod wired;

#[cfg_attr(target_family = "wasm", wasm_bindgen(getter_with_clone))]
#[derive(Default, Clone)]
pub struct RuntimeBackend {
    wired_scene: wired::scene::WiredSceneBackend,
}
