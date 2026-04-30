use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct WiredPortal {}

#[wasm_bindgen]
impl WiredPortal {
    pub fn list_portals(&self) {}
    pub fn open_portal(&self) {}
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct WiredPortalTypes {}

#[wasm_bindgen]
impl WiredPortalTypes {
    pub fn portal_close(&self) {}
    pub fn portal_destination(&self) {}
    pub fn portal_drop(&self) {}
    pub fn portal_id(&self) {}
    pub fn portal_new(&self) {}
    pub fn portal_rep(&self) {}
}
