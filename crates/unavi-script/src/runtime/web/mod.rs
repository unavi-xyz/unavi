use std::sync::{Arc, Mutex};

use wasm_bindgen::prelude::*;

use crate::runtime::{Runtime, shared::wired::scene::WiredSceneBackend};

mod wired;

#[wasm_bindgen(module = "/dist/runtime.js")]
unsafe extern "C" {
    pub async fn build_script(bytes: &[u8], name: &str, runtime: Runtime);
}
