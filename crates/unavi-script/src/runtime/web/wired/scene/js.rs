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
