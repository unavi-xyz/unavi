use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct WiredWds {}

#[wasm_bindgen]
impl WiredWds {
    pub fn get_wds(&self) {}
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct WiredWdsTypes {}

#[wasm_bindgen]
impl WiredWdsTypes {
    pub fn query_future_drop(&self) {}
    pub fn query_future_new(&self) {}
    pub fn query_future_poll(&self) {}
    pub fn query_future_rep(&self) {}
    pub fn read_future_drop(&self) {}
    pub fn read_future_new(&self) {}
    pub fn read_future_poll(&self) {}
    pub fn read_future_rep(&self) {}
    pub fn wds_drop(&self) {}
    pub fn wds_new(&self) {}
    pub fn wds_query(&self) {}
    pub fn wds_read(&self) {}
    pub fn wds_rep(&self) {}
}
