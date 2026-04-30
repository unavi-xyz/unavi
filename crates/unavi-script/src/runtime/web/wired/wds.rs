use wasm_bindgen::prelude::*;

use crate::runtime::Runtime;

#[wasm_bindgen]
pub struct WdsHandle;

#[wasm_bindgen]
pub struct QueryFutureHandle;

#[wasm_bindgen]
pub struct ReadFutureHandle;

#[wasm_bindgen]
impl WdsHandle {
    pub fn query(&self, _filter: JsValue) -> QueryFutureHandle {
        todo!()
    }

    pub fn read(&self, _record_id: Vec<u8>) -> ReadFutureHandle {
        todo!()
    }
}

#[wasm_bindgen]
impl QueryFutureHandle {
    pub fn poll(&self) -> JsValue {
        todo!()
    }
}

#[wasm_bindgen]
impl ReadFutureHandle {
    pub fn poll(&self) -> JsValue {
        todo!()
    }
}

#[wasm_bindgen]
impl Runtime {
    pub fn wired_wds_class(&self) -> JsValue {
        let js = JsValue::from(WdsHandle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    pub fn wired_query_future_class(&self) -> JsValue {
        let js = JsValue::from(QueryFutureHandle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    pub fn wired_read_future_class(&self) -> JsValue {
        let js = JsValue::from(ReadFutureHandle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    pub fn wired_wds_get_wds(&self) -> WdsHandle {
        todo!()
    }
}
