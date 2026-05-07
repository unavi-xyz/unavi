use std::sync::Arc;

use wasm_bindgen::prelude::*;

use crate::runtime::shared::{
    self, Api,
    wired::scene::node::{NodeCollider, NodeRigidBody},
};

use super::{material::MaterialHandle, mesh::MeshHandle, util::opt_rep};

#[wasm_bindgen]
pub struct NodeHandle {
    rep: u32,
    api: Arc<Api>,
}

impl NodeHandle {
    pub fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }

    pub fn rep(&self) -> u32 {
        self.rep
    }
}

impl Drop for NodeHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let _ = shared::wired::scene::node::on_drop(&self.api, self.rep);
        }
    }
}

#[wasm_bindgen]
impl NodeHandle {
    pub fn id(&self) -> String {
        shared::wired::scene::node::id(&self.api, self.rep).unwrap_or_default()
    }

    #[wasm_bindgen(js_name = "clone")]
    pub fn clone_node(&self) -> Self {
        let rep = shared::wired::scene::node::clone(&self.api, self.rep).unwrap_or(u32::MAX);
        NodeHandle::new(rep, Arc::clone(&self.api))
    }

    pub fn name(&self) -> Option<String> {
        shared::wired::scene::node::name(&self.api, self.rep).unwrap_or_default()
    }

    #[wasm_bindgen(js_name = "setName")]
    pub fn set_name(&self, value: Option<String>) {
        let _ = shared::wired::scene::node::set_name(&self.api, self.rep, value);
    }

    pub fn translation(&self) -> JsValue {
        let [x, y, z] =
            shared::wired::scene::node::translation(&self.api, self.rep).unwrap_or_default();
        js_sys::Array::of3(&x.into(), &y.into(), &z.into()).into()
    }

    #[wasm_bindgen(js_name = "setTranslation")]
    pub fn set_translation(&self, value: JsValue) {
        let arr = js_sys::Array::from(&value);
        let x = arr.get(0).as_f64().unwrap_or(0.0) as f32;
        let y = arr.get(1).as_f64().unwrap_or(0.0) as f32;
        let z = arr.get(2).as_f64().unwrap_or(0.0) as f32;
        let _ = shared::wired::scene::node::set_translation(&self.api, self.rep, [x, y, z]);
    }

    pub fn rotation(&self) -> JsValue {
        let [x, y, z, w] =
            shared::wired::scene::node::rotation(&self.api, self.rep).unwrap_or_default();
        js_sys::Array::of4(&x.into(), &y.into(), &z.into(), &w.into()).into()
    }

    #[wasm_bindgen(js_name = "setRotation")]
    pub fn set_rotation(&self, value: JsValue) {
        let arr = js_sys::Array::from(&value);
        let x = arr.get(0).as_f64().unwrap_or(0.0) as f32;
        let y = arr.get(1).as_f64().unwrap_or(0.0) as f32;
        let z = arr.get(2).as_f64().unwrap_or(0.0) as f32;
        let w = arr.get(3).as_f64().unwrap_or(1.0) as f32;
        let _ = shared::wired::scene::node::set_rotation(&self.api, self.rep, [x, y, z, w]);
    }

    pub fn scale(&self) -> JsValue {
        let [x, y, z] = shared::wired::scene::node::scale(&self.api, self.rep).unwrap_or_default();
        js_sys::Array::of3(&x.into(), &y.into(), &z.into()).into()
    }

    #[wasm_bindgen(js_name = "setScale")]
    pub fn set_scale(&self, value: JsValue) {
        let arr = js_sys::Array::from(&value);
        let x = arr.get(0).as_f64().unwrap_or(1.0) as f32;
        let y = arr.get(1).as_f64().unwrap_or(1.0) as f32;
        let z = arr.get(2).as_f64().unwrap_or(1.0) as f32;
        let _ = shared::wired::scene::node::set_scale(&self.api, self.rep, [x, y, z]);
    }

    pub fn transform(&self) -> JsValue {
        let t = shared::wired::scene::node::transform(&self.api, self.rep).unwrap_or_default();
        let obj = js_sys::Object::new();
        let tr = js_sys::Array::of3(
            &t.translation[0].into(),
            &t.translation[1].into(),
            &t.translation[2].into(),
        );
        let ro = js_sys::Array::of4(
            &t.rotation[0].into(),
            &t.rotation[1].into(),
            &t.rotation[2].into(),
            &t.rotation[3].into(),
        );
        let sc = js_sys::Array::of3(&t.scale[0].into(), &t.scale[1].into(), &t.scale[2].into());
        js_sys::Reflect::set(&obj, &"translation".into(), &tr).ok();
        js_sys::Reflect::set(&obj, &"rotation".into(), &ro).ok();
        js_sys::Reflect::set(&obj, &"scale".into(), &sc).ok();
        obj.into()
    }

    #[wasm_bindgen(js_name = "setTransform")]
    pub fn set_transform(&self, value: JsValue) {
        let tr_js = js_sys::Reflect::get(&value, &"translation".into()).unwrap_or_default();
        let ro_js = js_sys::Reflect::get(&value, &"rotation".into()).unwrap_or_default();
        let sc_js = js_sys::Reflect::get(&value, &"scale".into()).unwrap_or_default();

        let tr = js_sys::Array::from(&tr_js);
        let ro = js_sys::Array::from(&ro_js);
        let sc = js_sys::Array::from(&sc_js);

        let t = shared::wired::scene::node::NodeTransform {
            translation: [
                tr.get(0).as_f64().unwrap_or(0.0) as f32,
                tr.get(1).as_f64().unwrap_or(0.0) as f32,
                tr.get(2).as_f64().unwrap_or(0.0) as f32,
            ],
            rotation: [
                ro.get(0).as_f64().unwrap_or(0.0) as f32,
                ro.get(1).as_f64().unwrap_or(0.0) as f32,
                ro.get(2).as_f64().unwrap_or(0.0) as f32,
                ro.get(3).as_f64().unwrap_or(1.0) as f32,
            ],
            scale: [
                sc.get(0).as_f64().unwrap_or(1.0) as f32,
                sc.get(1).as_f64().unwrap_or(1.0) as f32,
                sc.get(2).as_f64().unwrap_or(1.0) as f32,
            ],
        };
        let _ = shared::wired::scene::node::set_transform(&self.api, self.rep, t);
    }

    #[wasm_bindgen(js_name = "globalTransform")]
    pub fn global_transform(&self) -> JsValue {
        let t =
            shared::wired::scene::node::global_transform(&self.api, self.rep).unwrap_or_default();
        let obj = js_sys::Object::new();
        let tr = js_sys::Array::of3(
            &t.translation[0].into(),
            &t.translation[1].into(),
            &t.translation[2].into(),
        );
        let ro = js_sys::Array::of4(
            &t.rotation[0].into(),
            &t.rotation[1].into(),
            &t.rotation[2].into(),
            &t.rotation[3].into(),
        );
        let sc = js_sys::Array::of3(&t.scale[0].into(), &t.scale[1].into(), &t.scale[2].into());
        js_sys::Reflect::set(&obj, &"translation".into(), &tr).ok();
        js_sys::Reflect::set(&obj, &"rotation".into(), &ro).ok();
        js_sys::Reflect::set(&obj, &"scale".into(), &sc).ok();
        obj.into()
    }

    pub fn parent(&self) -> Option<Self> {
        let rep = shared::wired::scene::node::parent(&self.api, self.rep).ok()??;
        Some(NodeHandle::new(rep, Arc::clone(&self.api)))
    }

    pub fn children(&self) -> JsValue {
        let Ok(reps) = shared::wired::scene::node::children(&self.api, self.rep) else {
            return js_sys::Array::new().into();
        };
        reps.into_iter()
            .map(|rep| JsValue::from(NodeHandle::new(rep, Arc::clone(&self.api))))
            .collect::<js_sys::Array>()
            .into()
    }

    #[wasm_bindgen(js_name = "addChild")]
    pub fn add_child(&self, child: &NodeHandle) {
        let _ = shared::wired::scene::node::add_child(&self.api, self.rep, child.rep);
    }

    #[wasm_bindgen(js_name = "removeChild")]
    pub fn remove_child(&self, child: &NodeHandle) {
        let _ = shared::wired::scene::node::remove_child(&self.api, self.rep, child.rep);
    }

    pub fn mesh(&self) -> Option<MeshHandle> {
        let rep = shared::wired::scene::node::mesh(&self.api, self.rep).ok()??;
        Some(MeshHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "setMesh")]
    pub fn set_mesh(&self, value: JsValue) {
        let _ = shared::wired::scene::node::set_mesh(&self.api, self.rep, opt_rep(&value));
    }

    pub fn material(&self) -> Option<MaterialHandle> {
        let rep = shared::wired::scene::node::material(&self.api, self.rep).ok()??;
        Some(MaterialHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "setMaterial")]
    pub fn set_material(&self, value: JsValue) {
        let _ = shared::wired::scene::node::set_material(&self.api, self.rep, opt_rep(&value));
    }

    pub async fn collider(&self) -> JsValue {
        let Ok(Some(c)) = shared::wired::scene::node::collider(&self.api, self.rep).await else {
            return JsValue::NULL;
        };
        let obj = js_sys::Object::new();
        let set = |k: &str, v: JsValue| js_sys::Reflect::set(&obj, &k.into(), &v).ok();
        match c {
            NodeCollider::Capsule { height, radius } => {
                set("type", "capsule".into());
                set("height", height.into());
                set("radius", radius.into());
            }
            NodeCollider::ConvexHull(points) => {
                set("type", "convex-hull".into());
                let arr: js_sys::Float32Array = points.as_slice().into();
                set("points", arr.into());
            }
            NodeCollider::Cuboid([x, y, z]) => {
                set("type", "cuboid".into());
                set("x", x.into());
                set("y", y.into());
                set("z", z.into());
            }
            NodeCollider::Cylinder { height, radius } => {
                set("type", "cylinder".into());
                set("height", height.into());
                set("radius", radius.into());
            }
            NodeCollider::Sphere(radius) => {
                set("type", "sphere".into());
                set("radius", radius.into());
            }
            NodeCollider::Trimesh { indices, vertices } => {
                set("type", "trimesh".into());
                let idx: js_sys::Uint32Array = indices.as_slice().into();
                let verts: js_sys::Float32Array = vertices.as_slice().into();
                set("indices", idx.into());
                set("vertices", verts.into());
            }
        }
        obj.into()
    }

    #[wasm_bindgen(js_name = "setCollider")]
    pub async fn set_collider(&self, value: JsValue) {
        if value.is_null() || value.is_undefined() {
            let _ = shared::wired::scene::node::set_collider(&self.api, self.rep, None).await;
            return;
        }
        let get_f32 = |k: &str| {
            js_sys::Reflect::get(&value, &k.into())
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as f32
        };
        let kind = js_sys::Reflect::get(&value, &"type".into())
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();
        let c = match kind.as_str() {
            "capsule" => Some(NodeCollider::Capsule {
                height: get_f32("height"),
                radius: get_f32("radius"),
            }),
            "cuboid" => Some(NodeCollider::Cuboid([
                get_f32("x"),
                get_f32("y"),
                get_f32("z"),
            ])),
            "cylinder" => Some(NodeCollider::Cylinder {
                height: get_f32("height"),
                radius: get_f32("radius"),
            }),
            "sphere" => Some(NodeCollider::Sphere(get_f32("radius"))),
            _ => None,
        };
        let _ = shared::wired::scene::node::set_collider(&self.api, self.rep, c).await;
    }

    #[wasm_bindgen(js_name = "rigidBody")]
    pub fn rigid_body(&self) -> JsValue {
        let Ok(Some(rb)) = shared::wired::scene::node::rigid_body(&self.api, self.rep) else {
            return JsValue::NULL;
        };
        match rb {
            NodeRigidBody::Dynamic => "dynamic".into(),
            NodeRigidBody::Fixed => "fixed".into(),
            NodeRigidBody::Kinematic => "kinematic".into(),
        }
    }

    #[wasm_bindgen(js_name = "setRigidBody")]
    pub fn set_rigid_body(&self, value: JsValue) {
        let rb = value.as_string().and_then(|s| match s.as_str() {
            "dynamic" => Some(NodeRigidBody::Dynamic),
            "fixed" => Some(NodeRigidBody::Fixed),
            "kinematic" => Some(NodeRigidBody::Kinematic),
            _ => None,
        });
        let _ = shared::wired::scene::node::set_rigid_body(&self.api, self.rep, rb);
    }
}
