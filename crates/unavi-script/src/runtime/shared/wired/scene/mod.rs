#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_family = "wasm", wasm_bindgen(getter_with_clone))]
#[derive(Default, Clone)]
pub struct WiredSceneBackend;

impl WiredSceneBackend {}
