use wasm_bindgen::JsValue;

use crate::error::ScriptError;

pub mod agent;
pub mod event;
pub mod input;
pub mod kv;
pub mod peer;
pub mod physics;
pub mod portal;
pub mod scene;
pub mod wds;

/// A WIT variant crosses into JS as a tag and an optional value, which is how
/// `jco` lowers one.
pub fn variant_obj(tag: &str, val: JsValue) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"tag".into(), &tag.into()).ok();
    js_sys::Reflect::set(&obj, &"val".into(), &val).ok();
    obj.into()
}

/// `wired:error/types.error`, lowered. Only `other` carries a payload, so the
/// detail on the structured variants stays host-side.
pub fn error_obj(err: &ScriptError) -> JsValue {
    match err {
        ScriptError::Other(detail) => variant_obj("other", detail.into()),
        ScriptError::QuotaFlow(_) => variant_obj("quota-flow", JsValue::UNDEFINED),
        ScriptError::QuotaStock(_) => variant_obj("quota-stock", JsValue::UNDEFINED),
        ScriptError::Policy(policy) if policy.is_permission() => {
            variant_obj("permission", JsValue::UNDEFINED)
        }
        ScriptError::Policy(_) => variant_obj("reach", JsValue::UNDEFINED),
    }
}
