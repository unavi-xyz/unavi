use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Clone)]
pub struct WiredPortal {}

#[wasm_bindgen]
impl WiredPortal {
    pub fn list_portals(&self) -> JsValue {
        todo!()
    }
    pub fn open_portal(&self, params: JsValue) -> JsValue {
        todo!()
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Clone)]
pub struct WiredPortalTypes {}

#[wasm_bindgen]
impl WiredPortalTypes {
    pub fn portal_close(&self, handle: JsValue) {}
    pub fn portal_destination(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn portal_drop(&self) {}
    pub fn portal_id(&self, handle: JsValue) -> String {
        todo!()
    }
    pub fn portal_new(&self) {}
    pub fn portal_rep(&self) {}
}
