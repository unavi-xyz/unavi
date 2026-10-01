use wasm_bindgen::{
    JsError,
    JsValue,
};

use crate::error::ScriptError;

pub mod agent;
pub mod event;
pub mod input;
pub mod peer;
pub mod physics;
pub mod portal;
pub mod scene;
pub mod storage;

/// A WIT variant crosses into JS as a tag and an optional value, which is how
/// `jco` lowers one. A case carrying nothing has no `val` key at all.
pub fn variant_obj(tag: &str, val: JsValue) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"tag".into(), &tag.into()).ok();
    if !val.is_undefined() {
        js_sys::Reflect::set(&obj, &"val".into(), &val).ok();
    }
    obj.into()
}

/// How a host function fails a WIT `result<_, error>`.
///
/// `jco`'s import shim catches whatever the function throws and lowers the
/// payload as the error, so the thrown value has to be the lowered variant
/// itself. A bare string traps the component where it is lowered, and a
/// sentinel handle returned in place of a throw reaches the guest as success.
pub fn raise(err: impl Into<ScriptError>) -> JsValue {
    error_obj(&err.into())
}

/// How a host function the WIT declares infallible fails: the exception passes
/// through `jco` uncaught and aborts the guest call, as a trap does natively.
/// Substituting a default in its place hands the guest a value it cannot tell
/// from a real one.
pub fn trap(err: &anyhow::Error) -> JsError {
    JsError::new(&format!("{err:#}"))
}

/// The same as `raise`, for a guest value that did not parse into what the
/// WIT declares.
pub fn malformed(detail: String) -> JsValue {
    error_obj(&ScriptError::Other(detail))
}

/// `wired:error/types.error`, lowered. Only `other` carries a payload, so the
/// detail on the structured variants stays host-side.
pub fn error_obj(err: &ScriptError) -> JsValue {
    match err {
        ScriptError::Other(detail) => variant_obj("other", detail.into()),
        ScriptError::QuotaFlow(_) => variant_obj("quota-flow", JsValue::UNDEFINED),
        ScriptError::QuotaStock(_) => variant_obj("quota-stock", JsValue::UNDEFINED),
        ScriptError::Permission(_) => variant_obj("permission", JsValue::UNDEFINED),
        ScriptError::NotOwner => variant_obj("forbidden", JsValue::UNDEFINED),
    }
}
