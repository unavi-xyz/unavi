use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct WiredInput {}

#[wasm_bindgen]
impl WiredInput {
    pub fn register_input_listener(&self) {}
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct WiredInputContext {}

#[wasm_bindgen]
impl WiredInputContext {
    pub fn listener(&self) {}
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct WiredInputTypes {}

#[wasm_bindgen]
impl WiredInputTypes {
    pub fn input_listener_drop(&self) {}
    pub fn input_listener_new(&self) {}
    pub fn input_listener_poll(&self) {}
    pub fn input_listener_rep(&self) {}
}
