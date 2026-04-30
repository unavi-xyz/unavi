use wasm_bindgen::prelude::*;

use crate::runtime::Runtime;

use super::scene::node::NodeHandle;

#[wasm_bindgen]
pub struct AgentHandle;

#[wasm_bindgen]
impl AgentHandle {
    pub fn bone(&self, _name: String) -> Option<NodeHandle> {
        todo!()
    }
}

#[wasm_bindgen]
impl Runtime {
    pub fn wired_agent_class(&self) -> JsValue {
        let js = JsValue::from(AgentHandle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    pub fn wired_agent_local_agent(&self) -> AgentHandle {
        todo!()
    }

    pub fn wired_agent_local_camera(&self) -> NodeHandle {
        todo!()
    }
}
