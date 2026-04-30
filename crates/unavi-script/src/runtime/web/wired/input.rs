use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Clone)]
pub struct WiredInput {}

#[wasm_bindgen]
impl WiredInput {
    pub fn register_input_listener(&self, target: JsValue) -> JsValue {
        todo!()
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Clone)]
pub struct WiredInputContext {}

#[wasm_bindgen]
impl WiredInputContext {
    pub fn listener(&self) -> JsValue {
        todo!()
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Clone)]
pub struct WiredInputTypes {}

#[wasm_bindgen]
impl WiredInputTypes {
    pub fn input_listener_drop(&self) {}
    pub fn input_listener_new(&self) {}
    pub fn input_listener_poll(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn input_listener_rep(&self) {}
}
