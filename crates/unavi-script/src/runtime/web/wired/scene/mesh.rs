use std::sync::Arc;

use wasm_bindgen::prelude::*;

use crate::runtime::shared::{
    self, Api,
    wired::scene::mesh::{MeshIndices, MeshTopology},
};

use super::js::{f32s_to_js, js_to_f32s};

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

#[wasm_bindgen]
impl MeshHandle {
    pub fn id(&self) -> String {
        shared::wired::scene::mesh::id(&self.api, self.rep).unwrap_or_default()
    }

    #[wasm_bindgen(js_name = "clone")]
    pub fn clone_mesh(&self) -> Self {
        let rep = shared::wired::scene::mesh::clone(&self.api, self.rep).unwrap_or(u32::MAX);
        Self::new(rep, self.api.clone())
    }

    pub fn name(&self) -> Option<String> {
        shared::wired::scene::mesh::name(&self.api, self.rep).unwrap_or_default()
    }

    pub fn set_name(&self, value: Option<String>) {
        let _ = shared::wired::scene::mesh::set_name(&self.api, self.rep, value);
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

    pub fn set_topology(&self, value: String) {
        let t = match value.as_str() {
            "point-list" => MeshTopology::PointList,
            "line-list" => MeshTopology::LineList,
            "line-strip" => MeshTopology::LineStrip,
            "triangle-list" => MeshTopology::TriangleList,
            "triangle-strip" => MeshTopology::TriangleStrip,
            _ => return,
        };
        let _ = shared::wired::scene::mesh::set_topology(&self.api, self.rep, t);
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

    pub async fn set_indices(&self, value: JsValue) {
        if value.is_null() || value.is_undefined() {
            let _ = shared::wired::scene::mesh::set_indices(&self.api, self.rep, None).await;
            return;
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
        let _ = shared::wired::scene::mesh::set_indices(&self.api, self.rep, indices).await;
    }

    pub async fn positions(&self) -> JsValue {
        f32s_to_js(shared::wired::scene::mesh::positions(&self.api, self.rep).await)
    }

    pub async fn set_positions(&self, value: JsValue) {
        let _ =
            shared::wired::scene::mesh::set_positions(&self.api, self.rep, js_to_f32s(value)).await;
    }

    pub async fn normals(&self) -> JsValue {
        f32s_to_js(shared::wired::scene::mesh::normals(&self.api, self.rep).await)
    }

    pub async fn set_normals(&self, value: JsValue) {
        let _ =
            shared::wired::scene::mesh::set_normals(&self.api, self.rep, js_to_f32s(value)).await;
    }

    pub async fn tangents(&self) -> JsValue {
        f32s_to_js(shared::wired::scene::mesh::tangents(&self.api, self.rep).await)
    }

    pub async fn set_tangents(&self, value: JsValue) {
        let _ =
            shared::wired::scene::mesh::set_tangents(&self.api, self.rep, js_to_f32s(value)).await;
    }

    pub async fn colors(&self) -> JsValue {
        f32s_to_js(shared::wired::scene::mesh::colors(&self.api, self.rep).await)
    }

    pub async fn set_colors(&self, value: JsValue) {
        let _ =
            shared::wired::scene::mesh::set_colors(&self.api, self.rep, js_to_f32s(value)).await;
    }

    pub async fn uv0(&self) -> JsValue {
        f32s_to_js(shared::wired::scene::mesh::uv0(&self.api, self.rep).await)
    }

    pub async fn set_uv0(&self, value: JsValue) {
        let _ = shared::wired::scene::mesh::set_uv0(&self.api, self.rep, js_to_f32s(value)).await;
    }

    pub async fn uv1(&self) -> JsValue {
        f32s_to_js(shared::wired::scene::mesh::uv1(&self.api, self.rep).await)
    }

    pub async fn set_uv1(&self, value: JsValue) {
        let _ = shared::wired::scene::mesh::set_uv1(&self.api, self.rep, js_to_f32s(value)).await;
    }
}

