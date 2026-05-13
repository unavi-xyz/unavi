use std::sync::Arc;

use wasm_bindgen::{JsValue, prelude::*};

use crate::runtime::shared::{
    self, Api,
    wired::scene::mesh::{MeshIndices, MeshTopology},
};

use super::util::{f32s_to_js, js_to_f32s};

fn indices_to_js(indices: MeshIndices) -> JsValue {
    let obj = js_sys::Object::new();
    let (tag, val): (&str, JsValue) = match indices {
        MeshIndices::Half(v) => ("half", js_sys::Uint16Array::from(v.as_slice()).into()),
        MeshIndices::Full(v) => ("full", js_sys::Uint32Array::from(v.as_slice()).into()),
    };
    js_sys::Reflect::set(&obj, &"tag".into(), &tag.into()).ok();
    js_sys::Reflect::set(&obj, &"val".into(), &val).ok();
    obj.into()
}

fn js_to_indices(value: &JsValue) -> Option<MeshIndices> {
    if value.is_null() || value.is_undefined() {
        return None;
    }
    let tag = js_sys::Reflect::get(value, &"tag".into())
        .ok()
        .and_then(|v| v.as_string())?;
    let val = js_sys::Reflect::get(value, &"val".into()).unwrap_or_default();
    match tag.as_str() {
        "half" => Some(MeshIndices::Half(js_sys::Uint16Array::new(&val).to_vec())),
        "full" => Some(MeshIndices::Full(js_sys::Uint32Array::new(&val).to_vec())),
        _ => None,
    }
}

#[wasm_bindgen]
pub struct MeshHandle {
    rep: u32,
    api: Arc<Api>,
}

impl MeshHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }

    pub const fn rep(&self) -> u32 {
        self.rep
    }
}

impl Drop for MeshHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let _ = shared::wired::scene::mesh::on_drop(&self.api, self.rep);
        }
    }
}

#[wasm_bindgen]
impl MeshHandle {
    pub fn id(&self) -> String {
        shared::wired::scene::mesh::id(&self.api, self.rep).unwrap_or_default()
    }

    #[wasm_bindgen(js_name = "__rep", getter)]
    #[expect(clippy::missing_const_for_fn)]
    pub fn wasm_rep(&self) -> u32 {
        self.rep
    }

    #[wasm_bindgen(js_name = "clone")]
    pub fn clone_mesh(&self) -> Self {
        let rep = shared::wired::scene::mesh::clone(&self.api, self.rep).unwrap_or(u32::MAX);
        Self::new(rep, Arc::clone(&self.api))
    }

    pub fn name(&self) -> Option<String> {
        shared::wired::scene::mesh::name(&self.api, self.rep).unwrap_or_default()
    }

    #[wasm_bindgen(js_name = "setName")]
    pub fn set_name(&self, value: Option<String>) -> Result<(), String> {
        shared::wired::scene::mesh::set_name(&self.api, self.rep, value).map_err(|e| e.to_string())
    }

    pub fn topology(&self) -> String {
        match shared::wired::scene::mesh::topology(&self.api, self.rep).unwrap_or_default() {
            MeshTopology::PointList => "point-list",
            MeshTopology::LineList => "line-list",
            MeshTopology::LineStrip => "line-strip",
            MeshTopology::TriangleList => "triangle-list",
            MeshTopology::TriangleStrip => "triangle-strip",
        }
        .into()
    }

    #[wasm_bindgen(js_name = "setTopology")]
    pub fn set_topology(&self, value: String) -> Result<(), String> {
        let t = match value.as_str() {
            "point-list" => MeshTopology::PointList,
            "line-list" => MeshTopology::LineList,
            "line-strip" => MeshTopology::LineStrip,
            "triangle-list" => MeshTopology::TriangleList,
            "triangle-strip" => MeshTopology::TriangleStrip,
            _ => return Ok(()),
        };
        shared::wired::scene::mesh::set_topology(&self.api, self.rep, t).map_err(|e| e.to_string())
    }

    pub async fn indices(&self) -> JsValue {
        shared::wired::scene::mesh::indices(&self.api, self.rep)
            .await
            .ok()
            .flatten()
            .map_or(JsValue::UNDEFINED, indices_to_js)
    }

    #[wasm_bindgen(js_name = "setIndices")]
    pub async fn set_indices(&self, value: JsValue) -> Result<(), String> {
        shared::wired::scene::mesh::set_indices(&self.api, self.rep, js_to_indices(&value))
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn positions(&self) -> JsValue {
        f32s_to_js(shared::wired::scene::mesh::positions(&self.api, self.rep).await)
    }

    #[wasm_bindgen(js_name = "setPositions")]
    pub async fn set_positions(&self, value: JsValue) -> Result<(), String> {
        shared::wired::scene::mesh::set_positions(&self.api, self.rep, js_to_f32s(value))
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn normals(&self) -> JsValue {
        f32s_to_js(shared::wired::scene::mesh::normals(&self.api, self.rep).await)
    }

    #[wasm_bindgen(js_name = "setNormals")]
    pub async fn set_normals(&self, value: JsValue) -> Result<(), String> {
        shared::wired::scene::mesh::set_normals(&self.api, self.rep, js_to_f32s(value))
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn tangents(&self) -> JsValue {
        f32s_to_js(shared::wired::scene::mesh::tangents(&self.api, self.rep).await)
    }

    #[wasm_bindgen(js_name = "setTangents")]
    pub async fn set_tangents(&self, value: JsValue) -> Result<(), String> {
        shared::wired::scene::mesh::set_tangents(&self.api, self.rep, js_to_f32s(value))
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn colors(&self) -> JsValue {
        f32s_to_js(shared::wired::scene::mesh::colors(&self.api, self.rep).await)
    }

    #[wasm_bindgen(js_name = "setColors")]
    pub async fn set_colors(&self, value: JsValue) -> Result<(), String> {
        shared::wired::scene::mesh::set_colors(&self.api, self.rep, js_to_f32s(value))
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn uv0(&self) -> JsValue {
        f32s_to_js(shared::wired::scene::mesh::uv0(&self.api, self.rep).await)
    }

    #[wasm_bindgen(js_name = "setUv0")]
    pub async fn set_uv0(&self, value: JsValue) -> Result<(), String> {
        shared::wired::scene::mesh::set_uv0(&self.api, self.rep, js_to_f32s(value))
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn uv1(&self) -> JsValue {
        f32s_to_js(shared::wired::scene::mesh::uv1(&self.api, self.rep).await)
    }

    #[wasm_bindgen(js_name = "setUv1")]
    pub async fn set_uv1(&self, value: JsValue) -> Result<(), String> {
        shared::wired::scene::mesh::set_uv1(&self.api, self.rep, js_to_f32s(value))
            .await
            .map_err(|e| e.to_string())
    }
}
