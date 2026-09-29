use hsd::attributes::xform::XformAttr;
use wasm_bindgen::JsValue;

pub fn vec3_to_js(x: f32, y: f32, z: f32) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"x".into(), &x.into()).ok();
    js_sys::Reflect::set(&obj, &"y".into(), &y.into()).ok();
    js_sys::Reflect::set(&obj, &"z".into(), &z.into()).ok();
    obj.into()
}

pub fn quat_to_js(x: f32, y: f32, z: f32, w: f32) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"x".into(), &x.into()).ok();
    js_sys::Reflect::set(&obj, &"y".into(), &y.into()).ok();
    js_sys::Reflect::set(&obj, &"z".into(), &z.into()).ok();
    js_sys::Reflect::set(&obj, &"w".into(), &w.into()).ok();
    obj.into()
}

pub fn js_to_vec3(v: &JsValue, default: [f32; 3]) -> [f32; 3] {
    let get = |k: &str, d: f32| {
        js_sys::Reflect::get(v, &k.into())
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or_else(|| f64::from(d)) as f32
    };
    [
        get("x", default[0]),
        get("y", default[1]),
        get("z", default[2]),
    ]
}

pub fn js_to_quat(v: &JsValue, default: [f32; 4]) -> [f32; 4] {
    let get = |k: &str, d: f32| {
        js_sys::Reflect::get(v, &k.into())
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or_else(|| f64::from(d)) as f32
    };
    [
        get("x", default[0]),
        get("y", default[1]),
        get("z", default[2]),
        get("w", default[3]),
    ]
}

pub fn js_to_f32s(value: JsValue) -> Option<Vec<f32>> {
    if value.is_null() || value.is_undefined() {
        return None;
    }
    Some(js_sys::Float32Array::new(&value).to_vec())
}

pub fn js_to_u32s(value: JsValue) -> Option<Vec<u32>> {
    if value.is_null() || value.is_undefined() {
        return None;
    }
    Some(js_sys::Uint32Array::new(&value).to_vec())
}

pub fn bytes32_to_js(b: &[u8; 32]) -> JsValue {
    js_sys::Uint8Array::from(b.as_slice()).into()
}

pub fn js_to_bytes32(value: &JsValue) -> Option<[u8; 32]> {
    if value.is_null() || value.is_undefined() {
        return None;
    }
    let arr = js_sys::Uint8Array::new(value).to_vec();
    arr.as_slice().try_into().ok()
}

/// Extracts the `SlotMap` index from a resource handle, exposed via its
/// `__rep` getter.
pub fn opt_rep(value: &JsValue) -> Option<u32> {
    if value.is_null() || value.is_undefined() {
        return None;
    }
    js_sys::Reflect::get(value, &"__rep".into())
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as u32)
}

pub fn obj_get(obj: &JsValue, key: &str) -> JsValue {
    js_sys::Reflect::get(obj, &key.into()).unwrap_or(JsValue::UNDEFINED)
}

pub fn obj_set(obj: &js_sys::Object, key: &str, value: &JsValue) {
    js_sys::Reflect::set(obj, &key.into(), value).ok();
}

pub fn obj_get_f32(obj: &JsValue, key: &str) -> Option<f32> {
    js_sys::Reflect::get(obj, &key.into())
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
}

pub fn obj_get_i32(obj: &JsValue, key: &str) -> Option<i32> {
    js_sys::Reflect::get(obj, &key.into())
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as i32)
}

pub fn obj_get_bool(obj: &JsValue, key: &str) -> Option<bool> {
    js_sys::Reflect::get(obj, &key.into())
        .ok()
        .and_then(|v| v.as_bool())
}

pub fn obj_get_string(obj: &JsValue, key: &str) -> Option<String> {
    js_sys::Reflect::get(obj, &key.into())
        .ok()
        .and_then(|v| v.as_string())
}

pub fn xform_to_js(x: &XformAttr) -> JsValue {
    let obj = js_sys::Object::new();
    obj_set(
        &obj,
        "translation",
        &vec3_to_js(x.translation[0], x.translation[1], x.translation[2]),
    );
    obj_set(
        &obj,
        "rotation",
        &quat_to_js(x.rotation[0], x.rotation[1], x.rotation[2], x.rotation[3]),
    );
    obj_set(
        &obj,
        "scale",
        &vec3_to_js(x.scale[0], x.scale[1], x.scale[2]),
    );
    obj.into()
}

pub fn js_to_xform(v: &JsValue) -> Option<XformAttr> {
    if v.is_null() || v.is_undefined() {
        return None;
    }
    Some(XformAttr {
        translation: js_to_vec3(&obj_get(v, "translation"), [0.0; 3]),
        rotation:    js_to_quat(&obj_get(v, "rotation"), [0.0, 0.0, 0.0, 1.0]),
        scale:       js_to_vec3(&obj_get(v, "scale"), [1.0; 3]),
    })
}
