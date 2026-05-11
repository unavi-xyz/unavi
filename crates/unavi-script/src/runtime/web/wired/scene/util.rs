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
            .unwrap_or(d as f64) as f32
    };
    [get("x", default[0]), get("y", default[1]), get("z", default[2])]
}

pub fn js_to_quat(v: &JsValue, default: [f32; 4]) -> [f32; 4] {
    let get = |k: &str, d: f32| {
        js_sys::Reflect::get(v, &k.into())
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(d as f64) as f32
    };
    [
        get("x", default[0]),
        get("y", default[1]),
        get("z", default[2]),
        get("w", default[3]),
    ]
}

pub fn f32s_to_js(result: anyhow::Result<Option<Vec<f32>>>) -> JsValue {
    match result {
        Ok(Some(v)) => js_sys::Float32Array::from(v.as_slice()).into(),
        _ => JsValue::NULL,
    }
}

pub fn js_to_f32s(value: JsValue) -> Option<Vec<f32>> {
    if value.is_null() || value.is_undefined() {
        return None;
    }
    Some(js_sys::Float32Array::new(&value).to_vec())
}

/// Extract the `__rep` resource-table index from an optional borrowed resource.
/// Returns `None` if the value is null, undefined, or missing `__rep`.
pub fn opt_rep(value: &JsValue) -> Option<u32> {
    if value.is_null() || value.is_undefined() {
        return None;
    }
    js_sys::Reflect::get(value, &"__rep".into())
        .ok()
        .and_then(|v| v.as_f64())
        .map(|v| v as u32)
}
