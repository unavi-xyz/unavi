//! `wired:peer/identity` and `wired:peer/authority`.

use wasm_bindgen::prelude::*;

use super::{
    Runtime,
    convert,
    scene::DocumentHandle,
};
use crate::host::peer;

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "selfDid")]
    pub fn self_did(&self) -> Result<String, JsValue> {
        peer::self_did(&self.host.borrow()).map_err(convert::raise)
    }

    pub fn owner(&self, document: &DocumentHandle) -> Result<JsValue, JsValue> {
        peer::owner(&self.host.borrow(), document.rep())
            .map(|did| did.map_or(JsValue::UNDEFINED, |did| JsValue::from_str(&did)))
            .map_err(convert::raise)
    }

    pub fn holder(&self, document: &DocumentHandle) -> Result<JsValue, JsValue> {
        peer::holder(&self.host.borrow(), document.rep())
            .map(|did| did.map_or(JsValue::UNDEFINED, |did| JsValue::from_str(&did)))
            .map_err(convert::raise)
    }

    #[wasm_bindgen(js_name = "isOwner")]
    pub fn is_owner(&self, document: &DocumentHandle) -> Result<bool, JsValue> {
        peer::is_owner(&self.host.borrow(), document.rep()).map_err(convert::raise)
    }

    #[wasm_bindgen(js_name = "isHolder")]
    pub fn is_holder(&self, document: &DocumentHandle) -> Result<bool, JsValue> {
        peer::is_holder(&self.host.borrow(), document.rep()).map_err(convert::raise)
    }

    #[wasm_bindgen(js_name = "takeHold")]
    pub fn take_hold(&self, document: &DocumentHandle) -> Result<(), JsValue> {
        peer::take_hold(&self.host.borrow(), document.rep()).map_err(convert::raise)
    }

    #[wasm_bindgen(js_name = "releaseHold")]
    pub fn release_hold(&self, document: &DocumentHandle, to: JsValue) -> Result<(), JsValue> {
        let to = to.as_string();
        peer::release_hold(&self.host.borrow(), document.rep(), to.as_deref())
            .map_err(convert::raise)
    }
}
