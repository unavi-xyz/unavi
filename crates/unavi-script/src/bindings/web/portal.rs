//! `wired:portal/portals`.

use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

use super::{
    HostHandle,
    Runtime,
    convert,
    scene::DocumentHandle,
};
use crate::{
    error::ScriptError,
    host::{
        portal,
        shared_state::link_intents::LinkIntent,
    },
};

/// A guest's open intent subscription. See `MessageSubscriptionHandle` in
/// `bindings::web::event` for how its `Drop` comes to run.
#[wasm_bindgen]
pub struct IntentSubscriptionHandle {
    rep:  u32,
    host: HostHandle,
}

impl Drop for IntentSubscriptionHandle {
    fn drop(&mut self) {
        let _ = portal::close(&mut self.host.borrow_mut(), self.rep);
    }
}

fn link_intent(value: JsValue) -> Result<LinkIntent, ScriptError> {
    #[derive(serde::Deserialize)]
    struct Pair {
        #[serde(rename = "sourceSpace")]
        source_space: (u64, u64, u64, u64),
        link:         (u64, u64),
    }
    let p: Pair = convert::from_js(value)?;
    Ok(LinkIntent {
        source_space: convert::doc_id_from_words(p.source_space),
        link:         convert::link_id_from_words(p.link),
    })
}

fn wit_link_intent(intent: LinkIntent) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &obj,
        &"sourceSpace".into(),
        &convert::wit_doc_id(intent.source_space),
    )
    .ok();
    js_sys::Reflect::set(&obj, &"link".into(), &convert::wit_link_id(intent.link)).ok();
    obj.into()
}

#[wasm_bindgen]
impl Runtime {
    pub fn open(
        &self,
        document: &DocumentHandle,
        prim: JsValue,
        target_space: JsValue,
    ) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let doc = document.rep();
        let parsed = (|| -> Result<_, ScriptError> {
            Ok((convert::prim_id(prim)?, convert::doc_id(target_space)?))
        })();
        let (prim, target) = match parsed {
            Ok(v) => v,
            Err(err) => return js_sys::Promise::reject(&convert::raise(err)),
        };
        future_to_promise(async move {
            let host = host.borrow();
            portal::open(&host, doc, prim, target)
                .await
                .map(|()| JsValue::UNDEFINED)
                .map_err(convert::raise)
        })
    }

    pub fn pair(
        &self,
        document: &DocumentHandle,
        prim: JsValue,
        intent: JsValue,
    ) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let doc = document.rep();
        let parsed =
            (|| -> Result<_, ScriptError> { Ok((convert::prim_id(prim)?, link_intent(intent)?)) })(
            );
        let (prim, intent) = match parsed {
            Ok(v) => v,
            Err(err) => return js_sys::Promise::reject(&convert::raise(err)),
        };
        future_to_promise(async move {
            let host = host.borrow();
            portal::pair(&host, doc, prim, intent)
                .await
                .map(|()| JsValue::UNDEFINED)
                .map_err(convert::raise)
        })
    }

    pub fn travel(&self, target_space: JsValue) -> js_sys::Promise {
        let host = Rc::clone(&self.host);
        let target = match convert::doc_id(target_space) {
            Ok(id) => id,
            Err(err) => return js_sys::Promise::reject(&convert::raise(err)),
        };
        future_to_promise(async move {
            let host = host.borrow();
            portal::travel(&host, target)
                .await
                .map(|()| JsValue::UNDEFINED)
                .map_err(convert::raise)
        })
    }

    pub fn intents(&self) -> Result<IntentSubscriptionHandle, JsValue> {
        let rep = portal::intents(&mut self.host.borrow_mut()).map_err(convert::raise)?;
        Ok(IntentSubscriptionHandle {
            rep,
            host: Rc::clone(&self.host),
        })
    }
}

#[wasm_bindgen]
impl IntentSubscriptionHandle {
    pub fn drain(&self, max: u32) -> Result<Vec<JsValue>, JsValue> {
        portal::drain(&mut self.host.borrow_mut(), self.rep, max)
            .map(|intents| intents.into_iter().map(wit_link_intent).collect())
            .map_err(convert::into_trap)
    }

    pub fn dropped(&self) -> Result<u64, JsValue> {
        portal::dropped(&self.host.borrow(), self.rep).map_err(convert::into_trap)
    }

    pub fn claim(&self, intent: JsValue) -> Result<bool, JsValue> {
        let intent = link_intent(intent).map_err(convert::into_trap)?;
        portal::claim(&self.host.borrow(), self.rep, intent).map_err(convert::into_trap)
    }
}
