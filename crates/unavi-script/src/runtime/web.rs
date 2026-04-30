use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[wasm_bindgen(module = "/dist/runtime.js")]
unsafe extern "C" {
    pub async fn build_script(bytes: &[u8], name: &str);
}
