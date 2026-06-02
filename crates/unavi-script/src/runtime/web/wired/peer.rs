use wasm_bindgen::prelude::*;

use crate::runtime::{
    Runtime,
    shared,
};

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredPeerSelfPeer")]
    pub fn wired_peer_self_peer(&self) -> Vec<u8> {
        shared::wired::peer::self_peer(&self.api)
    }

    #[wasm_bindgen(js_name = "wiredPeerDocOwner")]
    pub fn wired_peer_doc_owner(&self, doc: Vec<u8>) -> JsValue {
        match shared::wired::peer::doc_owner(&self.api, doc) {
            Some(bytes) => js_sys::Uint8Array::from(bytes.as_slice()).into(),
            None => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "wiredPeerIsSelfOwner")]
    pub fn wired_peer_is_self_owner(&self) -> bool {
        shared::wired::peer::is_self_owner(&self.api)
    }
}
