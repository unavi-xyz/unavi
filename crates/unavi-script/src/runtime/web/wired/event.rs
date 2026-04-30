use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct WiredEvent {}

#[wasm_bindgen]
impl WiredEvent {
    pub fn emit(&self) {}
    pub fn listen(&self) {}
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct WiredEventTypes {}

#[wasm_bindgen]
impl WiredEventTypes {
    pub fn event_receptor_drop(&self) {}
    pub fn event_receptor_new(&self) {}
    pub fn event_receptor_poll(&self) {}
    pub fn event_receptor_rep(&self) {}
}
