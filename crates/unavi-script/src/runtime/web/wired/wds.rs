use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Clone)]
pub struct WiredWds {}

#[wasm_bindgen]
impl WiredWds {
    pub fn get_wds(&self) -> JsValue {
        todo!()
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Clone)]
pub struct WiredWdsTypes {}

#[wasm_bindgen]
impl WiredWdsTypes {
    pub fn query_future_drop(&self) {}
    pub fn query_future_new(&self) {}
    pub fn query_future_poll(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn query_future_rep(&self) {}
    pub fn read_future_drop(&self) {}
    pub fn read_future_new(&self) {}
    pub fn read_future_poll(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn read_future_rep(&self) {}
    pub fn wds_drop(&self) {}
    pub fn wds_new(&self) {}
    pub fn wds_query(&self, handle: JsValue, filter: JsValue) -> JsValue {
        todo!()
    }
    pub fn wds_read(&self, handle: JsValue, record_id: Vec<u8>) -> JsValue {
        todo!()
    }
    pub fn wds_rep(&self) {}
}
