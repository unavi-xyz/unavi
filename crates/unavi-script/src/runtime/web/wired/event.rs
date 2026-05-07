use wasm_bindgen::prelude::*;

use crate::runtime::Runtime;

#[wasm_bindgen]
pub struct EventReceptorHandle;

#[wasm_bindgen]
impl EventReceptorHandle {
    pub fn poll(&self) -> JsValue {
        todo!()
    }
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredEventReceptorClass")]
    pub fn wired_event_receptor_class(&self) -> JsValue {
        let js = JsValue::from(EventReceptorHandle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredEventEmit")]
    pub fn wired_event_emit(&self, _channel: String, _payload: Vec<u8>, _filter: JsValue) {
        todo!()
    }

    #[wasm_bindgen(js_name = "wiredEventListen")]
    pub fn wired_event_listen(&self, _channels: JsValue, _filter: JsValue) -> EventReceptorHandle {
        todo!()
    }
}
