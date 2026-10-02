//! `wired:scene/document`. `wired:scene/properties` has no `Host` methods of
//! its own; its types are built and read in [`super::convert`].

use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use super::{
    HostHandle,
    Runtime,
    convert,
};
use crate::{
    error::ScriptError,
    host::scene::{
        self,
        documents,
        edit,
        place,
    },
};

/// A guest's handle to a document, returned directly from the call that
/// minted it. Dropping a guest's own handle to it runs jco's drop
/// trampoline, which calls `Symbol.dispose` on the captured JS value;
/// `wasm-bindgen` wires every exported class's `Symbol.dispose` to its
/// `free()`, which runs [`Drop`] here — deterministic, not dependent on JS
/// garbage collection, and needing no entry in `runtime.ts`'s import object.
#[wasm_bindgen]
pub struct DocumentHandle {
    rep:  u32,
    host: HostHandle,
}

impl DocumentHandle {
    const fn new(rep: u32, host: HostHandle) -> Self {
        Self { rep, host }
    }

    /// The handle table index other binding modules pass through to
    /// `host::scene::*`, for calls that take a document (physics, input,
    /// portals, peer authority, agent attachment).
    pub(super) const fn rep(&self) -> u32 {
        self.rep
    }
}

impl Drop for DocumentHandle {
    fn drop(&mut self) {
        let _ = scene::drop_document(&mut self.host.borrow_mut(), self.rep);
    }
}

/// A `Promise` already rejected with `err`, for a conversion that fails
/// before the async body of a `js_sys::Promise`-returning method runs.
fn promise_err(err: JsValue) -> js_sys::Promise {
    js_sys::Promise::reject(&err)
}

#[wasm_bindgen]
impl DocumentHandle {
    pub fn id(&self) -> Result<JsValue, JsValue> {
        scene::id(&self.host.borrow(), self.rep)
            .map(convert::wit_doc_id)
            .map_err(convert::into_trap)
    }

    pub fn owned(&self) -> Result<bool, JsValue> {
        scene::owned(&self.host.borrow(), self.rep).map_err(convert::into_trap)
    }

    pub fn contains(&self, prim: JsValue) -> Result<bool, JsValue> {
        let prim = convert::prim_id(prim).map_err(convert::into_trap)?;
        scene::contains(&self.host.borrow(), self.rep, prim).map_err(convert::into_trap)
    }

    pub fn roots(&self) -> Result<Vec<JsValue>, JsValue> {
        scene::roots(&self.host.borrow(), self.rep)
            .map(|ids| ids.into_iter().map(convert::wit_prim_id).collect())
            .map_err(convert::into_trap)
    }

    pub fn prims(&self) -> Result<Vec<JsValue>, JsValue> {
        scene::prims(&self.host.borrow(), self.rep)
            .map(|ids| ids.into_iter().map(convert::wit_prim_id).collect())
            .map_err(convert::into_trap)
    }

    pub fn children(&self, prim: JsValue) -> Result<Vec<JsValue>, JsValue> {
        let prim = convert::prim_id(prim).map_err(convert::into_trap)?;
        scene::children(&self.host.borrow(), self.rep, prim)
            .map(|ids| ids.into_iter().map(convert::wit_prim_id).collect())
            .map_err(convert::into_trap)
    }

    #[wasm_bindgen(js_name = "findByName")]
    pub fn find_by_name(&self, name: String) -> Result<Vec<JsValue>, JsValue> {
        scene::find_by_name(&self.host.borrow(), self.rep, &name)
            .map(|ids| ids.into_iter().map(convert::wit_prim_id).collect())
            .map_err(convert::into_trap)
    }

    pub fn get(&self, prim: JsValue, key: JsValue) -> Result<JsValue, JsValue> {
        let prim = convert::prim_id(prim).map_err(convert::into_trap)?;
        let Ok(key) = convert::property_key(&key) else {
            // A key naming nothing a script could have written has no
            // value, matching native.
            return Ok(JsValue::UNDEFINED);
        };
        scene::get(&self.host.borrow(), self.rep, prim, &key)
            .map(|value| value.map_or(JsValue::UNDEFINED, convert::wit_property))
            .map_err(convert::into_trap)
    }

    #[wasm_bindgen(js_name = "getMany")]
    pub fn get_many(&self, keys: JsValue) -> Result<Vec<JsValue>, JsValue> {
        let keys = js_sys::Array::from(&keys);
        let parsed = keys
            .iter()
            .map(|entry| {
                let pair = js_sys::Array::from(&entry);
                let prim = convert::prim_id(pair.get(0)).ok()?;
                let key = convert::property_key(&pair.get(1)).ok()?;
                Some((prim, key))
            })
            .collect::<Vec<_>>();
        let known = parsed.iter().flatten().cloned().collect::<Vec<_>>();
        let mut values = scene::get_many(&self.host.borrow(), self.rep, &known)
            .map_err(convert::into_trap)?
            .into_iter();
        Ok(parsed
            .into_iter()
            .map(|key| {
                key.and_then(|_| values.next().flatten())
                    .map_or(JsValue::UNDEFINED, convert::wit_property)
            })
            .collect())
    }

    pub fn keys(&self, prim: JsValue) -> Result<Vec<JsValue>, JsValue> {
        let prim = convert::prim_id(prim).map_err(convert::into_trap)?;
        scene::keys(&self.host.borrow(), self.rep, prim)
            .map(|keys| keys.into_iter().map(convert::wit_property_key).collect())
            .map_err(convert::into_trap)
    }

    #[wasm_bindgen(js_name = "worldTransform")]
    pub fn world_transform(&self, prim: JsValue) -> Result<JsValue, JsValue> {
        let prim = convert::prim_id(prim).map_err(convert::raise)?;
        scene::world_transform(&self.host.borrow(), self.rep, prim)
            .map(convert::wit_transform)
            .map_err(convert::raise)
    }

    #[wasm_bindgen(js_name = "offsetTo")]
    pub fn offset_to(&self, other: &Self) -> Result<JsValue, JsValue> {
        scene::offset_to(&self.host.borrow(), self.rep, other.rep)
            .map(|t| t.map_or(JsValue::UNDEFINED, convert::wit_transform))
            .map_err(convert::into_trap)
    }

    #[wasm_bindgen(js_name = "createPrim")]
    pub fn create_prim(&self, layer: String, parent: JsValue) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let rep = self.rep;
        let parent = if parent.is_undefined() {
            None
        } else {
            match convert::prim_id(parent) {
                Ok(id) => Some(id),
                Err(err) => return promise_err(convert::raise(err)),
            }
        };
        future_to_promise(async move {
            let host = host.borrow();
            edit::create_prim(&host, rep, convert::layer(&layer), parent)
                .await
                .map(convert::wit_prim_id)
                .map_err(convert::raise)
        })
    }

    pub fn apply(&self, layer: String, edits: JsValue) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let rep = self.rep;
        let edits = match js_sys::Array::from(&edits)
            .iter()
            .map(convert::edit)
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(edits) => edits,
            Err(err) => return promise_err(convert::raise(err)),
        };
        future_to_promise(async move {
            if edits.len() > edit::MAX_EDITS {
                return Err(convert::raise(ScriptError::invalid(
                    "at most 4096 edits per apply",
                )));
            }
            let host = host.borrow();
            edit::apply(&host, rep, convert::layer(&layer), edits)
                .await
                .map(|()| JsValue::UNDEFINED)
                .map_err(convert::raise)
        })
    }

    pub fn commit(&self, keys: JsValue) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let rep = self.rep;
        let keys = js_sys::Array::from(&keys)
            .iter()
            .map(|entry| {
                let pair = js_sys::Array::from(&entry);
                Ok((
                    convert::prim_id(pair.get(0))?,
                    convert::property_key(&pair.get(1))?,
                ))
            })
            .collect::<Result<Vec<_>, _>>();
        let keys = match keys {
            Ok(keys) => keys,
            Err(err) => return promise_err(convert::raise(err)),
        };
        future_to_promise(async move {
            let host = host.borrow();
            edit::commit(&host, rep, keys)
                .await
                .map(|()| JsValue::UNDEFINED)
                .map_err(convert::raise)
        })
    }

    pub fn place(&self, anchor: JsValue, offset: JsValue) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let rep = self.rep;
        let placed =
            convert::anchor(anchor).and_then(|anchor| Ok((anchor, convert::xform(offset)?)));
        let (anchor, offset) = match placed {
            Ok(v) => v,
            Err(err) => return promise_err(convert::raise(err)),
        };
        future_to_promise(async move {
            let host = host.borrow();
            place::place(&host, rep, anchor, offset)
                .await
                .map(|()| JsValue::UNDEFINED)
                .map_err(convert::raise)
        })
    }
}

#[wasm_bindgen]
impl Runtime {
    #[wasm_bindgen(js_name = "scriptDocument")]
    pub fn script_document(&self) -> Result<DocumentHandle, JsValue> {
        scene::script_document(&mut self.host.borrow_mut())
            .map(|rep| DocumentHandle::new(rep, Rc::clone(&self.host)))
            .map_err(convert::raise)
    }

    #[wasm_bindgen(js_name = "scriptPrim")]
    pub fn script_prim(&self) -> JsValue {
        convert::wit_prim_id(scene::script_prim(&self.host.borrow()))
    }

    #[wasm_bindgen(js_name = "openDocument")]
    pub fn open_document(&self, id: JsValue) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let id = match convert::doc_id(id) {
            Ok(id) => id,
            Err(err) => return promise_err(convert::raise(err)),
        };
        future_to_promise(async move {
            let rep = {
                let mut host_mut = host.borrow_mut();
                scene::open_document(&mut host_mut, id).await
            }
            .map_err(convert::raise)?;
            Ok(rep.map_or(JsValue::UNDEFINED, |rep| {
                JsValue::from(DocumentHandle::new(rep, Rc::clone(&host)))
            }))
        })
    }

    #[wasm_bindgen(js_name = "createDocument")]
    pub fn create_document(&self) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        future_to_promise(async move {
            let rep = {
                let mut host_mut = host.borrow_mut();
                documents::create_document(&mut host_mut).await
            }
            .map_err(convert::raise)?;
            Ok(JsValue::from(DocumentHandle::new(rep, Rc::clone(&host))))
        })
    }

    #[wasm_bindgen(js_name = "copyDocument")]
    pub fn copy_document(&self, source: &DocumentHandle) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let source_rep = source.rep;
        future_to_promise(async move {
            let rep = {
                let mut host_mut = host.borrow_mut();
                documents::copy_document(&mut host_mut, source_rep).await
            }
            .map_err(convert::raise)?;
            Ok(JsValue::from(DocumentHandle::new(rep, Rc::clone(&host))))
        })
    }

    #[wasm_bindgen(js_name = "deleteDocument")]
    pub fn delete_document(&self, document: DocumentHandle) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        future_to_promise(async move {
            // Kept alive until the call completes: `delete_document` removes
            // the table entry itself, so `document`'s own `Drop` afterward is
            // a harmless no-op, but dropping it before the call would make
            // the entry it needs to read look like an invalid handle.
            let rep = document.rep;
            let result = {
                let mut host_mut = host.borrow_mut();
                documents::delete_document(&mut host_mut, rep).await
            };
            drop(document);
            result.map(|()| JsValue::UNDEFINED).map_err(convert::raise)
        })
    }
}
