use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(module = "/dist/runtime.js")]
unsafe extern "C" {
    fn build_script(bytes: &[u8], name: &str) -> String;
}
