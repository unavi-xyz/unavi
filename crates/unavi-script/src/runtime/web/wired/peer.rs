use unavi_policy::document::ApiName;
use wasm_bindgen::prelude::*;

use super::raise;
use crate::runtime::{
    Runtime,
    shared,
};

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "wiredPeerSelfPeer")]
    pub fn wired_peer_self_peer(&self) -> Result<JsValue, JsValue> {
        self.api.require(ApiName::Identity).map_err(raise)?;
        Ok(
            shared::wired::peer::self_peer(&self.api).map_or(JsValue::UNDEFINED, |bytes| {
                js_sys::Uint8Array::from(bytes.as_slice()).into()
            }),
        )
    }

    #[wasm_bindgen(js_name = "wiredPeerSelfDid")]
    pub fn wired_peer_self_did(&self) -> Result<JsValue, JsValue> {
        self.api.require(ApiName::Identity).map_err(raise)?;
        Ok(shared::wired::peer::self_did(&self.api)
            .map_or(JsValue::UNDEFINED, |did| JsValue::from_str(&did)))
    }

    #[wasm_bindgen(js_name = "wiredPeerDocOwner")]
    pub fn wired_peer_doc_owner(&self, doc: Vec<u8>) -> Result<JsValue, JsValue> {
        self.api.require(ApiName::Peer).map_err(raise)?;
        Ok(
            shared::wired::peer::doc_owner(&self.api, doc).map_or(JsValue::UNDEFINED, |bytes| {
                js_sys::Uint8Array::from(bytes.as_slice()).into()
            }),
        )
    }

    #[wasm_bindgen(js_name = "wiredPeerIsSelfOwner")]
    pub fn wired_peer_is_self_owner(&self) -> Result<bool, JsValue> {
        self.api.require(ApiName::Peer).map_err(raise)?;
        Ok(shared::wired::peer::is_self_owner(&self.api))
    }
}
