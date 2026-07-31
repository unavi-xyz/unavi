use wasm_bindgen::prelude::*;

use crate::{
    permissions::ApiName,
    runtime::{
        Runtime,
        shared,
    },
};

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredPeerSelfPeer")]
    #[must_use]
    pub fn wired_peer_self_peer(&self) -> JsValue {
        if self.api.require(ApiName::Peer).is_err() {
            return JsValue::UNDEFINED;
        }
        shared::wired::peer::self_peer(&self.api).map_or(JsValue::UNDEFINED, |bytes| {
            js_sys::Uint8Array::from(bytes.as_slice()).into()
        })
    }

    #[wasm_bindgen(js_name = "wiredPeerSelfDid")]
    #[must_use]
    pub fn wired_peer_self_did(&self) -> JsValue {
        if self.api.require(ApiName::Peer).is_err() {
            return JsValue::UNDEFINED;
        }
        shared::wired::peer::self_did(&self.api)
            .map_or(JsValue::UNDEFINED, |did| JsValue::from_str(&did))
    }

    #[wasm_bindgen(js_name = "wiredPeerDocOwner")]
    #[must_use]
    pub fn wired_peer_doc_owner(&self, doc: Vec<u8>) -> JsValue {
        if self.api.require(ApiName::Peer).is_err() {
            return JsValue::UNDEFINED;
        }
        shared::wired::peer::doc_owner(&self.api, doc).map_or(JsValue::UNDEFINED, |bytes| {
            js_sys::Uint8Array::from(bytes.as_slice()).into()
        })
    }

    #[wasm_bindgen(js_name = "wiredPeerIsSelfOwner")]
    #[must_use]
    pub fn wired_peer_is_self_owner(&self) -> bool {
        self.api.require(ApiName::Peer).is_ok() && shared::wired::peer::is_self_owner(&self.api)
    }
}
