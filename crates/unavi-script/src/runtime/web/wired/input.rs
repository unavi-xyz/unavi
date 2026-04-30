use wasm_bindgen::prelude::*;

use crate::runtime::Runtime;

use super::scene::node::NodeHandle;

#[wasm_bindgen]
pub struct InputListenerHandle;

#[wasm_bindgen]
impl InputListenerHandle {
    pub fn poll(&self) -> JsValue {
        todo!()
    }
}

#[wasm_bindgen]
impl Runtime {
    pub fn wired_input_listener_class(&self) -> JsValue {
        let js = JsValue::from(InputListenerHandle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    pub fn wired_input_register_input_listener(&self, _target: NodeHandle) -> InputListenerHandle {
        todo!()
    }

    pub fn wired_input_context_listener(&self) -> InputListenerHandle {
        todo!()
    }
}
