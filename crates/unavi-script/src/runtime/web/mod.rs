use wasm_bindgen::prelude::*;

use crate::runtime::Runtime;

mod wired;

#[wasm_bindgen(module = "/dist/runtime.js")]
unsafe extern "C" {
    pub async fn build_script(bytes: &[u8], name: &str, runtime: Runtime);
}
