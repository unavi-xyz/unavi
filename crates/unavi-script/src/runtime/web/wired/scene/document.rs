use std::sync::Arc;

use tokio::sync::Mutex;
use wasm_bindgen::prelude::*;

use crate::runtime::shared::wired::scene::WiredSceneBackend;

use super::{material::MaterialHandle, mesh::MeshHandle, node::NodeHandle};

#[wasm_bindgen]
pub struct DocHandle {
    rep: u32,
    backend: Arc<Mutex<WiredSceneBackend>>,
}

impl DocHandle {
    pub const fn new(rep: u32, backend: Arc<Mutex<WiredSceneBackend>>) -> Self {
        Self { rep, backend }
    }
}

#[wasm_bindgen]
impl DocHandle {
    pub async fn id(&self) -> Vec<u8> {
        self.backend
            .lock()
            .await
            .doc_id(self.rep)
            .unwrap_or_default()
    }

    #[wasm_bindgen(js_name = "clone")]
    pub async fn clone_doc(&self) -> Option<DocHandle> {
        let rep = self.backend.lock().await.doc_clone(self.rep)?;
        Some(DocHandle::new(rep, Arc::clone(&self.backend)))
    }

    pub async fn nodes(&self) -> js_sys::Array {
        let reps = self
            .backend
            .lock()
            .await
            .doc_nodes(self.rep)
            .await
            .unwrap_or_default();
        reps.into_iter()
            .map(|rep| JsValue::from(NodeHandle::new(rep, Arc::clone(&self.backend))))
            .collect()
    }

    pub async fn roots(&self) -> js_sys::Array {
        let reps = self
            .backend
            .lock()
            .await
            .doc_roots(self.rep)
            .await
            .unwrap_or_default();
        reps.into_iter()
            .map(|rep| JsValue::from(NodeHandle::new(rep, Arc::clone(&self.backend))))
            .collect()
    }

    pub async fn meshes(&self) -> js_sys::Array {
        let reps = self
            .backend
            .lock()
            .await
            .doc_meshes(self.rep)
            .await
            .unwrap_or_default();
        reps.into_iter()
            .map(|rep| JsValue::from(MeshHandle::new(rep, Arc::clone(&self.backend))))
            .collect()
    }

    pub async fn materials(&self) -> js_sys::Array {
        let reps = self
            .backend
            .lock()
            .await
            .doc_materials(self.rep)
            .await
            .unwrap_or_default();
        reps.into_iter()
            .map(|rep| JsValue::from(MaterialHandle::new(rep, Arc::clone(&self.backend))))
            .collect()
    }

    pub async fn create_node(&self) -> NodeHandle {
        let rep = self
            .backend
            .lock()
            .await
            .doc_create_node(self.rep)
            .await
            .expect("create node");
        NodeHandle::new(rep, Arc::clone(&self.backend))
    }

    pub fn create_mesh(&self) -> MeshHandle {
        let rep = self
            .backend
            .try_lock()
            .expect("no contention")
            .doc_create_mesh(self.rep)
            .expect("create mesh");
        MeshHandle::new(rep, Arc::clone(&self.backend))
    }

    pub fn create_material(&self) -> MaterialHandle {
        let rep = self
            .backend
            .try_lock()
            .expect("no contention")
            .doc_create_material(self.rep)
            .expect("create material");
        MaterialHandle::new(rep, Arc::clone(&self.backend))
    }

    pub fn remove_node(&self, value: NodeHandle) {
        self.backend
            .try_lock()
            .expect("no contention")
            .doc_remove_node(value.rep());
    }

    pub fn remove_mesh(&self, value: MeshHandle) {
        self.backend
            .try_lock()
            .expect("no contention")
            .doc_remove_mesh(value.rep());
    }

    pub fn remove_material(&self, value: MaterialHandle) {
        self.backend
            .try_lock()
            .expect("no contention")
            .doc_remove_material(value.rep());
    }

    pub fn translation(&self) -> Vec<f32> {
        let v = self
            .backend
            .try_lock()
            .expect("no contention")
            .doc_translation(self.rep)
            .unwrap_or_default();
        vec![v.x, v.y, v.z]
    }

    pub fn set_translation(&self, value: Vec<f32>) {
        let mut guard = self.backend.try_lock().expect("no contention");
        let mut t = guard.doc_transform(self.rep).unwrap_or_default();
        t.translation = bevy::math::Vec3::new(
            value.first().copied().unwrap_or(0.0),
            value.get(1).copied().unwrap_or(0.0),
            value.get(2).copied().unwrap_or(0.0),
        );
        let _ = guard.doc_set_transform(self.rep, t);
    }

    pub fn rotation(&self) -> Vec<f32> {
        let q = self
            .backend
            .try_lock()
            .expect("no contention")
            .doc_rotation(self.rep)
            .unwrap_or(bevy::math::Quat::IDENTITY);
        vec![q.x, q.y, q.z, q.w]
    }

    pub fn set_rotation(&self, value: Vec<f32>) {
        let mut guard = self.backend.try_lock().expect("no contention");
        let mut t = guard.doc_transform(self.rep).unwrap_or_default();
        t.rotation = bevy::math::Quat::from_xyzw(
            value.first().copied().unwrap_or(0.0),
            value.get(1).copied().unwrap_or(0.0),
            value.get(2).copied().unwrap_or(0.0),
            value.get(3).copied().unwrap_or(1.0),
        );
        let _ = guard.doc_set_transform(self.rep, t);
    }

    pub fn scale(&self) -> Vec<f32> {
        let v = self
            .backend
            .try_lock()
            .expect("no contention")
            .doc_scale(self.rep)
            .unwrap_or(bevy::math::Vec3::ONE);
        vec![v.x, v.y, v.z]
    }

    pub fn set_scale(&self, value: Vec<f32>) {
        let mut guard = self.backend.try_lock().expect("no contention");
        let mut t = guard.doc_transform(self.rep).unwrap_or_default();
        t.scale = bevy::math::Vec3::new(
            value.first().copied().unwrap_or(1.0),
            value.get(1).copied().unwrap_or(1.0),
            value.get(2).copied().unwrap_or(1.0),
        );
        let _ = guard.doc_set_transform(self.rep, t);
    }

    pub fn transform(&self) -> Vec<f32> {
        let t = self
            .backend
            .try_lock()
            .expect("no contention")
            .doc_transform(self.rep)
            .unwrap_or_default();
        vec![
            t.translation.x,
            t.translation.y,
            t.translation.z,
            t.rotation.x,
            t.rotation.y,
            t.rotation.z,
            t.rotation.w,
            t.scale.x,
            t.scale.y,
            t.scale.z,
        ]
    }

    pub fn set_transform(&self, value: Vec<f32>) {
        if value.len() < 10 {
            return;
        }
        let t = bevy::transform::components::Transform {
            translation: bevy::math::Vec3::new(value[0], value[1], value[2]),
            rotation: bevy::math::Quat::from_xyzw(value[3], value[4], value[5], value[6]),
            scale: bevy::math::Vec3::new(value[7], value[8], value[9]),
        };
        let _ = self
            .backend
            .try_lock()
            .expect("no contention")
            .doc_set_transform(self.rep, t);
    }

    pub fn global_transform(&self) -> Vec<f32> {
        let gt = self
            .backend
            .try_lock()
            .expect("no contention")
            .doc_global_transform(self.rep)
            .unwrap_or_default();
        let (scale, rotation, translation) = gt.to_scale_rotation_translation();
        vec![
            translation.x,
            translation.y,
            translation.z,
            rotation.x,
            rotation.y,
            rotation.z,
            rotation.w,
            scale.x,
            scale.y,
            scale.z,
        ]
    }

    pub fn sync(&self) -> bool {
        false
    }

    pub fn set_sync(&self, _value: bool) {}

    pub fn public(&self) -> bool {
        false
    }

    pub fn set_public(&self, _value: bool) {}

    pub fn assets(&self) -> js_sys::Array {
        js_sys::Array::new()
    }

    pub fn add_asset(&self, _name: String, _blob_id: Vec<u8>) {}

    pub fn remove_asset(&self, _name: String) {}
}
