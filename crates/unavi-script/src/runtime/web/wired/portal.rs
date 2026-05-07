use wasm_bindgen::prelude::*;

use crate::runtime::Runtime;

#[wasm_bindgen]
pub struct PortalHandle;

#[wasm_bindgen]
impl PortalHandle {
    pub fn close(&self) {
        todo!()
    }

    pub fn destination(&self) -> JsValue {
        todo!()
    }

    pub fn id(&self) -> String {
        todo!()
    }
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredPortalClass")]
    pub fn wired_portal_class(&self) -> JsValue {
        let js = JsValue::from(PortalHandle);
        js_sys::Reflect::get(&js, &JsValue::from_str("constructor")).expect("reflect")
    }

    #[wasm_bindgen(js_name = "wiredPortalListPortals")]
    pub fn wired_portal_list_portals(&self) -> JsValue {
        todo!()
    }

    #[wasm_bindgen(js_name = "wiredPortalOpenPortal")]
    pub fn wired_portal_open_portal(&self, _params: JsValue) -> JsValue {
        todo!()
    }
}
