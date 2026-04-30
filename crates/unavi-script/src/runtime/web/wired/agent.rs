use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct WiredAgent {}

#[wasm_bindgen]
impl WiredAgent {
    pub fn local_agent(&self) -> JsValue { todo!() }
    pub fn local_camera(&self) -> JsValue { todo!() }
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct WiredAgentTypes {}

#[wasm_bindgen]
impl WiredAgentTypes {
    pub fn agent_bone(&self, handle: JsValue, name: String) -> JsValue { todo!() }
    pub fn agent_drop(&self) {}
    pub fn agent_new(&self) {}
    pub fn agent_rep(&self) {}
}
