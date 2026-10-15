//! `unavi:host/node-storage`.

use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use super::{
    HostHandle,
    Runtime,
    convert,
};
use crate::host::storage;

/// A guest's pending single-value read. See `MessageSubscriptionHandle` in
/// `bindings::web::event` for how its `Drop` comes to run.
#[wasm_bindgen]
pub struct PendingValueHandle {
    rep:  u32,
    host: HostHandle,
}

impl Drop for PendingValueHandle {
    fn drop(&mut self) {
        let _ = self.host.borrow_mut().pending_values.remove(self.rep);
    }
}

/// A guest's pending listing read. See `MessageSubscriptionHandle` in
/// `bindings::web::event` for how its `Drop` comes to run.
#[wasm_bindgen]
pub struct PendingEntriesHandle {
    rep:  u32,
    host: HostHandle,
}

impl Drop for PendingEntriesHandle {
    fn drop(&mut self) {
        let _ = self.host.borrow_mut().pending_entries.remove(self.rep);
    }
}

/// The nested `result<T, error>` a poll answers once ready: unlike a call's
/// own `result<_, error>`, this one is a plain value, not a thrown exception,
/// so it is built as the `{tag, val}` shape directly.
fn wire_result<T>(
    result: Result<T, crate::error::ScriptError>,
    ok: impl FnOnce(T) -> JsValue,
) -> JsValue {
    match result {
        Ok(value) => tagged("ok", ok(value)),
        // `convert::raise` lowers the same `error` shape this result's `err`
        // case holds; a pending read never fails with `InvalidHandle`, so
        // `raise`'s trap arm for it is unreachable here.
        Err(err) => tagged("err", convert::raise(err)),
    }
}

fn tagged(tag: &str, val: JsValue) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &JsValue::from_str("tag"), &JsValue::from_str(tag)).ok();
    js_sys::Reflect::set(&obj, &JsValue::from_str("val"), &val).ok();
    obj.into()
}

#[wasm_bindgen]
impl PendingValueHandle {
    pub fn poll(&self) -> JsValue {
        let Some(result) = self
            .host
            .borrow()
            .pending_values
            .get(self.rep)
            .ok()
            .map(storage::Pending::poll)
        else {
            return JsValue::UNDEFINED;
        };
        let Some(result) = result else {
            return JsValue::UNDEFINED;
        };
        wire_result(result, |value| {
            value.map_or(JsValue::UNDEFINED, |bytes| {
                js_sys::Uint8Array::from(bytes.as_slice()).into()
            })
        })
    }
}

#[wasm_bindgen]
impl PendingEntriesHandle {
    pub fn poll(&self) -> JsValue {
        let Some(result) = self
            .host
            .borrow()
            .pending_entries
            .get(self.rep)
            .ok()
            .map(storage::Pending::poll)
        else {
            return JsValue::UNDEFINED;
        };
        let Some(result) = result else {
            return JsValue::UNDEFINED;
        };
        wire_result(result, |entries| {
            entries
                .into_iter()
                .map(|(key, value)| {
                    let obj = js_sys::Object::new();
                    js_sys::Reflect::set(&obj, &"key".into(), &JsValue::from_str(&key)).ok();
                    js_sys::Reflect::set(
                        &obj,
                        &"value".into(),
                        &js_sys::Uint8Array::from(value.as_slice()),
                    )
                    .ok();
                    JsValue::from(obj)
                })
                .collect::<js_sys::Array>()
                .into()
        })
    }
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "rootDocument")]
    pub fn root_document(&self) -> Result<JsValue, JsValue> {
        storage::root_document(&self.host.borrow())
            .map(|id| id.map_or(JsValue::UNDEFINED, convert::wit_doc_id))
            .map_err(convert::raise)
    }

    pub fn registries(&self) -> Result<Vec<JsValue>, JsValue> {
        storage::registries(&self.host.borrow())
            .map(|ids| ids.into_iter().map(convert::wit_doc_id).collect())
            .map_err(convert::raise)
    }

    pub fn get(&self, document: JsValue, key: String) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let doc = match convert::doc_id(document) {
            Ok(id) => id,
            Err(err) => return js_sys::Promise::reject(&convert::raise(err)),
        };
        future_to_promise(async move {
            let rep = {
                let mut host_mut = host.borrow_mut();
                storage::get(&mut host_mut, doc, key).await
            }
            .map_err(convert::raise)?;
            Ok(JsValue::from(PendingValueHandle {
                rep,
                host: Rc::clone(&host),
            }))
        })
    }

    #[wasm_bindgen(js_name = "listEntries")]
    pub fn list_entries(&self, document: JsValue, prefix: String, limit: u32) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let doc = match convert::doc_id(document) {
            Ok(id) => id,
            Err(err) => return js_sys::Promise::reject(&convert::raise(err)),
        };
        future_to_promise(async move {
            let rep = {
                let mut host_mut = host.borrow_mut();
                storage::list_entries(&mut host_mut, doc, prefix, limit).await
            }
            .map_err(convert::raise)?;
            Ok(JsValue::from(PendingEntriesHandle {
                rep,
                host: Rc::clone(&host),
            }))
        })
    }
}
