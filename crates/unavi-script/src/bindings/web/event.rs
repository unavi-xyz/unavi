//! `wired:event/messaging`.

use std::rc::Rc;

use wasm_bindgen::prelude::*;

use super::{
    HostHandle,
    Runtime,
    convert,
};
use crate::{
    error::ScriptError,
    host::event::{
        self,
        MAX_FILTER_DOCUMENTS,
    },
};

/// A guest's open message subscription. `wasm-bindgen` wires every exported
/// class's `Symbol.dispose` to its `free()`, which runs this `Drop` and
/// closes the listener; jco's own drop trampoline calls it directly on the
/// captured value, needing no entry in `runtime.ts`'s import object.
#[wasm_bindgen]
pub struct MessageSubscriptionHandle {
    rep:  u32,
    host: HostHandle,
}

impl Drop for MessageSubscriptionHandle {
    fn drop(&mut self) {
        let _ = event::close(&mut self.host.borrow_mut(), self.rep);
    }
}

fn docs(value: &JsValue) -> Result<Option<Vec<hsd::id::DocId>>, ScriptError> {
    if value.is_undefined() {
        return Ok(None);
    }
    let array = js_sys::Array::from(value);
    if array.length() as usize > MAX_FILTER_DOCUMENTS {
        return Err(ScriptError::invalid("a filter names at most 64 documents"));
    }
    array
        .iter()
        .map(convert::doc_id)
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

/// `wired:event/messaging.scope`. `spatial.origin` is a `prim-ref`, which
/// carries ids `serde` cannot lower (a WIT `u64` as a JS `bigint`), so it is
/// read field by field rather than through one `serde`-derived shape.
fn scope(value: &JsValue) -> Result<event::Scope, ScriptError> {
    match convert::tag(value).as_str() {
        "global" => Ok(event::Scope::Global),
        "spatial" => {
            let val = convert::val(value);
            let origin = js_sys::Reflect::get(&val, &JsValue::from_str("origin"))
                .map_err(|_| ScriptError::invalid("a spatial scope names its origin"))?;
            let (doc, prim) = convert::prim_ref(origin)?;
            let radius = js_sys::Reflect::get(&val, &JsValue::from_str("radius"))
                .ok()
                .and_then(|v| v.as_f64())
                .ok_or_else(|| ScriptError::invalid("a spatial scope names its radius"))?
                as f32;
            Ok(event::Scope::Spatial { doc, prim, radius })
        }
        _ => Err(ScriptError::invalid("not a scope")),
    }
}

#[wasm_bindgen]
impl Runtime {
    pub fn emit(
        &self,
        channel: String,
        payload: JsValue,
        audience: JsValue,
        scope: JsValue,
    ) -> Result<(), JsValue> {
        let payload = js_sys::Uint8Array::new(&payload).to_vec();
        let audience = docs(&audience).map_err(convert::raise)?;
        let scope = self::scope(&scope).map_err(convert::raise)?;
        event::emit(&self.host.borrow(), &channel, payload, audience, scope).map_err(convert::raise)
    }

    pub fn listen(
        &self,
        channels: JsValue,
        senders: JsValue,
        scope: JsValue,
    ) -> Result<MessageSubscriptionHandle, JsValue> {
        let channels = js_sys::Array::from(&channels)
            .iter()
            .map(|v| {
                v.as_string()
                    .ok_or_else(|| ScriptError::invalid("a channel name is a string"))
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(convert::raise)?;
        let senders = docs(&senders).map_err(convert::raise)?;
        let scope = self::scope(&scope).map_err(convert::raise)?;
        let rep = event::listen(&mut self.host.borrow_mut(), channels, senders, scope)
            .map_err(convert::raise)?;
        Ok(MessageSubscriptionHandle {
            rep,
            host: Rc::clone(&self.host),
        })
    }
}

#[wasm_bindgen]
impl MessageSubscriptionHandle {
    pub fn drain(&self, max: u32) -> Result<Vec<JsValue>, JsValue> {
        event::drain(&mut self.host.borrow_mut(), self.rep, max)
            .map(|messages| messages.into_iter().map(wit_message).collect())
            .map_err(convert::into_trap)
    }

    pub fn dropped(&self) -> Result<u64, JsValue> {
        event::dropped(&self.host.borrow(), self.rep).map_err(convert::into_trap)
    }

    pub fn claim(&self, token: u64) -> Result<bool, JsValue> {
        event::claim(&self.host.borrow(), self.rep, token).map_err(convert::into_trap)
    }
}

fn wit_message(message: event::Message) -> JsValue {
    let obj = js_sys::Object::new();
    let set = |key: &str, value: &JsValue| {
        js_sys::Reflect::set(&obj, &JsValue::from_str(key), value).ok();
    };
    set("channel", &JsValue::from_str(&message.channel));
    set(
        "payload",
        &js_sys::Uint8Array::from(message.payload.as_slice()),
    );
    set("sender", &convert::wit_doc_id(message.sender));
    set(
        "distance",
        &message
            .distance
            .map_or(JsValue::UNDEFINED, |d| JsValue::from_f64(f64::from(d))),
    );
    set("sentAt", &JsValue::from_f64(message.sent_at));
    set(
        "claimToken",
        &message
            .claim_token
            .map_or(JsValue::UNDEFINED, JsValue::from),
    );
    obj.into()
}
