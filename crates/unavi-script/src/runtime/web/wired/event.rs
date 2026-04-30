use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct WiredEvent {}

#[wasm_bindgen]
impl WiredEvent {
    pub fn emit(&self, channel: String, payload: Vec<u8>, filter: JsValue) {}
    pub fn listen(&self, channels: JsValue, filter: JsValue) -> JsValue { todo!() }
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct WiredEventTypes {}

#[wasm_bindgen]
impl WiredEventTypes {
    pub fn event_receptor_drop(&self) {}
    pub fn event_receptor_new(&self) {}
    pub fn event_receptor_poll(&self, handle: JsValue) -> JsValue { todo!() }
    pub fn event_receptor_rep(&self) {}
}
