use bevy::prelude::Transform;
use js_sys::{Object, Reflect, Uint32Array};
use wasm_bindgen::{JsCast, JsValue};

pub fn js_vec3(x: f32, y: f32, z: f32) -> JsValue {
    let obj = Object::new();
    Reflect::set(&obj, &"x".into(), &JsValue::from_f64(f64::from(x))).ok();
    Reflect::set(&obj, &"y".into(), &JsValue::from_f64(f64::from(y))).ok();
    Reflect::set(&obj, &"z".into(), &JsValue::from_f64(f64::from(z))).ok();
    obj.into()
}

pub fn js_quat(x: f32, y: f32, z: f32, w: f32) -> JsValue {
    let obj = Object::new();
    Reflect::set(&obj, &"x".into(), &JsValue::from_f64(f64::from(x))).ok();
    Reflect::set(&obj, &"y".into(), &JsValue::from_f64(f64::from(y))).ok();
    Reflect::set(&obj, &"z".into(), &JsValue::from_f64(f64::from(z))).ok();
    Reflect::set(&obj, &"w".into(), &JsValue::from_f64(f64::from(w))).ok();
    obj.into()
}

pub fn js_transform(t: &Transform) -> JsValue {
    let tr = js_vec3(t.translation.x, t.translation.y, t.translation.z);
    let ro = js_quat(t.rotation.x, t.rotation.y, t.rotation.z, t.rotation.w);
    let sc = js_vec3(t.scale.x, t.scale.y, t.scale.z);
    let obj = Object::new();
    Reflect::set(&obj, &"translation".into(), &tr).ok();
    Reflect::set(&obj, &"rotation".into(), &ro).ok();
    Reflect::set(&obj, &"scale".into(), &sc).ok();
    obj.into()
}

pub fn js_u32_array(v: &[u32]) -> JsValue {
    let arr = Uint32Array::new_with_length(v.len() as u32);
    arr.copy_from(v);
    arr.into()
}

pub fn js_f32_array(v: &[f32]) -> JsValue {
    let arr = js_sys::Float32Array::new_with_length(v.len() as u32);
    arr.copy_from(v);
    arr.into()
}

pub fn f32_from_js(v: &JsValue) -> Option<Vec<f32>> {
    if v.is_null() || v.is_undefined() {
        return None;
    }
    let fa = v.dyn_ref::<js_sys::Float32Array>()?;
    Some(fa.to_vec())
}

pub fn js_string_array(v: &JsValue) -> Option<Vec<String>> {
    let arr = v.dyn_ref::<js_sys::Array>()?;
    let mut out = Vec::with_capacity(arr.length() as usize);
    for i in 0..arr.length() {
        out.push(arr.get(i).as_string()?);
    }
    Some(out)
}

pub fn parse_f32_array(v: &JsValue) -> Vec<f32> {
    v.dyn_ref::<js_sys::Float32Array>()
        .map(|fa| fa.to_vec())
        .unwrap_or_default()
}

pub fn parse_js_doc_list(v: &JsValue) -> Option<Vec<blake3::Hash>> {
    if v.is_null() || v.is_undefined() {
        return None;
    }
    let arr = v.dyn_ref::<js_sys::Array>()?;
    let mut out = Vec::with_capacity(arr.length() as usize);
    for i in 0..arr.length() {
        let bytes: [u8; 32] = arr
            .get(i)
            .dyn_ref::<js_sys::Uint8Array>()?
            .to_vec()
            .try_into()
            .ok()?;
        out.push(blake3::Hash::from(bytes));
    }
    Some(out)
}
