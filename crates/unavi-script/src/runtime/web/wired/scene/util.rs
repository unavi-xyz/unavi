use wasm_bindgen::JsValue;

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
