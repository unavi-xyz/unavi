use std::sync::Arc;

use wasm_bindgen::prelude::*;

use crate::runtime::shared::{
    self, Api,
    wired::scene::mesh::{MeshIndices, MeshTopology},
};

use super::util::{f32s_to_js, js_to_f32s};

#[wasm_bindgen]
pub struct MeshHandle {
    rep: u32,
    api: Arc<Api>,
}

impl MeshHandle {
    pub fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }

    pub fn rep(&self) -> u32 {
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
    pub fn wasm_rep(&self) -> u32 {
        self.rep
    }

    #[wasm_bindgen(js_name = "clone")]
    pub fn clone_mesh(&self) -> Self {
        let rep = shared::wired::scene::mesh::clone(&self.api, self.rep).unwrap_or(u32::MAX);
        Self::new(rep, self.api.clone())
    }

    pub fn name(&self) -> Option<String> {
        shared::wired::scene::mesh::name(&self.api, self.rep).unwrap_or_default()
    }

    #[wasm_bindgen(js_name = "setName")]
    pub fn set_name(&self, value: Option<String>) -> Result<(), String> {
        shared::wired::scene::mesh::set_name(&self.api, self.rep, value)
            .map_err(|e| e.to_string())
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
        shared::wired::scene::mesh::set_topology(&self.api, self.rep, t)
            .map_err(|e| e.to_string())
    }

    pub async fn indices(&self) -> JsValue {
        let Ok(Some(indices)) = shared::wired::scene::mesh::indices(&self.api, self.rep).await
        else {
            return JsValue::NULL;
        };
        let obj = js_sys::Object::new();
        match indices {
            MeshIndices::Half(v) => {
                let arr: js_sys::Uint16Array = v.as_slice().into();
                js_sys::Reflect::set(&obj, &"type".into(), &"half".into()).unwrap();
                js_sys::Reflect::set(&obj, &"data".into(), &arr.into()).unwrap();
            }
            MeshIndices::Full(v) => {
                let arr: js_sys::Uint32Array = v.as_slice().into();
                js_sys::Reflect::set(&obj, &"type".into(), &"full".into()).unwrap();
                js_sys::Reflect::set(&obj, &"data".into(), &arr.into()).unwrap();
            }
        }
        obj.into()
    }

    #[wasm_bindgen(js_name = "setIndices")]
    pub async fn set_indices(&self, value: JsValue) -> Result<(), String> {
        if value.is_null() || value.is_undefined() {
            return shared::wired::scene::mesh::set_indices(&self.api, self.rep, None)
                .await
                .map_err(|e| e.to_string());
        }
        let get = |k: &str| js_sys::Reflect::get(&value, &k.into()).ok();
        let kind = get("type").and_then(|v| v.as_string()).unwrap_or_default();
        let data = get("data").unwrap_or(JsValue::UNDEFINED);
        let indices = match kind.as_str() {
            "half" => {
                let arr = js_sys::Uint16Array::new(&data);
                Some(MeshIndices::Half(arr.to_vec()))
            }
            "full" => {
                let arr = js_sys::Uint32Array::new(&data);
                Some(MeshIndices::Full(arr.to_vec()))
            }
            _ => None,
        };
        shared::wired::scene::mesh::set_indices(&self.api, self.rep, indices)
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
