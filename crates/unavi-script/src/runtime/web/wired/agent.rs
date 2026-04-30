use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct WiredAgent {}

#[wasm_bindgen]
impl WiredAgent {
    pub fn local_agent(&self) {}
    pub fn local_camera(&self) {}
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct WiredAgentTypes {}

#[wasm_bindgen]
impl WiredAgentTypes {
    pub fn agent_bone(&self) {}
    pub fn agent_new(&self) {}
    pub fn agent_rep(&self) {}
    pub fn agent_drop(&self) {}
}
