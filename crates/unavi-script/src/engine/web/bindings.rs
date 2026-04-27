#[wasm_bindgen(module = "/assets/unavi-script/runtime.js")]
extern "C" {
    fn build_script(bytes: &[u8], name: &str) -> String;
}
