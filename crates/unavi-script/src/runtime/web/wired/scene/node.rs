use std::sync::Arc;

use wasm_bindgen::{JsValue, prelude::*};

use crate::runtime::shared::{
    self, Api,
    wired::scene::node::{NodeCollider, NodeRigidBody, NodeTransform},
};

use super::{
    material::MaterialHandle,
    mesh::MeshHandle,
    util::{js_to_quat, js_to_vec3, opt_rep, quat_to_js, vec3_to_js},
};

fn transform_to_js(t: &NodeTransform) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(
        &obj,
        &"translation".into(),
        &vec3_to_js(t.translation[0], t.translation[1], t.translation[2]),
    )
    .ok();
    js_sys::Reflect::set(
        &obj,
        &"rotation".into(),
        &quat_to_js(t.rotation[0], t.rotation[1], t.rotation[2], t.rotation[3]),
    )
    .ok();
    js_sys::Reflect::set(
        &obj,
        &"scale".into(),
        &vec3_to_js(t.scale[0], t.scale[1], t.scale[2]),
    )
    .ok();
    obj.into()
}

fn js_to_transform(v: &JsValue) -> NodeTransform {
    let get = |k: &str| js_sys::Reflect::get(v, &k.into()).unwrap_or_default();
    NodeTransform {
        translation: js_to_vec3(&get("translation"), [0.0; 3]),
        rotation: js_to_quat(&get("rotation"), [0.0, 0.0, 0.0, 1.0]),
        scale: js_to_vec3(&get("scale"), [1.0; 3]),
    }
}

fn collider_to_js(c: NodeCollider) -> JsValue {
    let variant = |tag: &str, val: JsValue| -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &"tag".into(), &tag.into()).ok();
        js_sys::Reflect::set(&obj, &"val".into(), &val).ok();
        obj.into()
    };
    let record2 = |k1: &str, v1: f32, k2: &str, v2: f32| -> JsValue {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &k1.into(), &v1.into()).ok();
        js_sys::Reflect::set(&obj, &k2.into(), &v2.into()).ok();
        obj.into()
    };
    match c {
        NodeCollider::Capsule { height, radius } => {
            variant("capsule", record2("height", height, "radius", radius))
        }
        NodeCollider::ConvexHull(points) => {
            variant("convex-hull", js_sys::Float32Array::from(points.as_slice()).into())
        }
        NodeCollider::Cuboid([x, y, z]) => variant("cuboid", vec3_to_js(x, y, z)),
        NodeCollider::Cylinder { height, radius } => {
            variant("cylinder", record2("height", height, "radius", radius))
        }
        NodeCollider::Sphere(radius) => variant("sphere", radius.into()),
        NodeCollider::Trimesh { indices, vertices } => {
            let val = js_sys::Object::new();
            js_sys::Reflect::set(
                &val,
                &"indices".into(),
                &js_sys::Uint32Array::from(indices.as_slice()).into(),
            )
            .ok();
            js_sys::Reflect::set(
                &val,
                &"vertices".into(),
                &js_sys::Float32Array::from(vertices.as_slice()).into(),
            )
            .ok();
            variant("trimesh", val.into())
        }
    }
}

fn js_to_collider(value: &JsValue) -> Option<NodeCollider> {
    if value.is_null() || value.is_undefined() {
        return None;
    }
    let tag = js_sys::Reflect::get(value, &"tag".into())
        .ok()
        .and_then(|v| v.as_string())?;
    let val = js_sys::Reflect::get(value, &"val".into()).unwrap_or_default();
    let get_f32 = |obj: &JsValue, k: &str| {
        js_sys::Reflect::get(obj, &k.into())
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0) as f32
    };
    Some(match tag.as_str() {
        "capsule" => NodeCollider::Capsule {
            height: get_f32(&val, "height"),
            radius: get_f32(&val, "radius"),
        },
        "convex-hull" => NodeCollider::ConvexHull(js_sys::Float32Array::new(&val).to_vec()),
        "cuboid" => NodeCollider::Cuboid(js_to_vec3(&val, [0.0; 3])),
        "cylinder" => NodeCollider::Cylinder {
            height: get_f32(&val, "height"),
            radius: get_f32(&val, "radius"),
        },
        "sphere" => NodeCollider::Sphere(val.as_f64().unwrap_or(0.0) as f32),
        "trimesh" => NodeCollider::Trimesh {
            indices: js_sys::Uint32Array::new(
                &js_sys::Reflect::get(&val, &"indices".into()).unwrap_or_default(),
            )
            .to_vec(),
            vertices: js_sys::Float32Array::new(
                &js_sys::Reflect::get(&val, &"vertices".into()).unwrap_or_default(),
            )
            .to_vec(),
        },
        _ => return None,
    })
}

#[wasm_bindgen]
pub struct NodeHandle {
    rep: u32,
    api: Arc<Api>,
}

impl NodeHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }

    pub const fn rep(&self) -> u32 {
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
    #[wasm_bindgen(js_name = "__rep", getter)]
    #[expect(clippy::missing_const_for_fn)]
    pub fn wasm_rep(&self) -> u32 {
        self.rep
    }

    pub fn id(&self) -> String {
        shared::wired::scene::node::id(&self.api, self.rep).unwrap_or_default()
    }

    #[wasm_bindgen(js_name = "clone")]
    pub fn clone_node(&self) -> Self {
        let rep = shared::wired::scene::node::clone(&self.api, self.rep).unwrap_or(u32::MAX);
        Self::new(rep, Arc::clone(&self.api))
    }

    pub fn name(&self) -> Option<String> {
        shared::wired::scene::node::name(&self.api, self.rep).unwrap_or_default()
    }

    #[wasm_bindgen(js_name = "setName")]
    pub fn set_name(&self, value: Option<String>) -> Result<(), String> {
        shared::wired::scene::node::set_name(&self.api, self.rep, value).map_err(|e| e.to_string())
    }

    pub fn translation(&self) -> JsValue {
        let [x, y, z] =
            shared::wired::scene::node::translation(&self.api, self.rep).unwrap_or_default();
        vec3_to_js(x, y, z)
    }

    #[wasm_bindgen(js_name = "setTranslation")]
    pub fn set_translation(&self, value: JsValue) -> Result<(), String> {
        shared::wired::scene::node::set_translation(
            &self.api,
            self.rep,
            js_to_vec3(&value, [0.0; 3]),
        )
        .map_err(|e| e.to_string())
    }

    pub fn rotation(&self) -> JsValue {
        let [x, y, z, w] =
            shared::wired::scene::node::rotation(&self.api, self.rep).unwrap_or_default();
        quat_to_js(x, y, z, w)
    }

    #[wasm_bindgen(js_name = "setRotation")]
    pub fn set_rotation(&self, value: JsValue) -> Result<(), String> {
        shared::wired::scene::node::set_rotation(
            &self.api,
            self.rep,
            js_to_quat(&value, [0.0, 0.0, 0.0, 1.0]),
        )
        .map_err(|e| e.to_string())
    }

    pub fn scale(&self) -> JsValue {
        let [x, y, z] = shared::wired::scene::node::scale(&self.api, self.rep).unwrap_or_default();
        vec3_to_js(x, y, z)
    }

    #[wasm_bindgen(js_name = "setScale")]
    pub fn set_scale(&self, value: JsValue) -> Result<(), String> {
        shared::wired::scene::node::set_scale(
            &self.api,
            self.rep,
            js_to_vec3(&value, [1.0; 3]),
        )
        .map_err(|e| e.to_string())
    }

    pub fn transform(&self) -> JsValue {
        let t = shared::wired::scene::node::transform(&self.api, self.rep).unwrap_or_default();
        transform_to_js(&t)
    }

    #[wasm_bindgen(js_name = "setTransform")]
    pub fn set_transform(&self, value: JsValue) -> Result<(), String> {
        shared::wired::scene::node::set_transform(&self.api, self.rep, js_to_transform(&value))
            .map_err(|e| e.to_string())
    }

    #[wasm_bindgen(js_name = "globalTransform")]
    pub fn global_transform(&self) -> JsValue {
        let t =
            shared::wired::scene::node::global_transform(&self.api, self.rep).unwrap_or_default();
        transform_to_js(&t)
    }

    pub fn parent(&self) -> Option<Self> {
        let rep = shared::wired::scene::node::parent(&self.api, self.rep).ok()??;
        Some(Self::new(rep, Arc::clone(&self.api)))
    }

    pub fn children(&self) -> JsValue {
        let Ok(reps) = shared::wired::scene::node::children(&self.api, self.rep) else {
            return js_sys::Array::new().into();
        };
        reps.into_iter()
            .map(|rep| JsValue::from(Self::new(rep, Arc::clone(&self.api))))
            .collect::<js_sys::Array>()
            .into()
    }

    #[wasm_bindgen(js_name = "addChild")]
    pub fn add_child(&self, child: &Self) -> Result<(), String> {
        shared::wired::scene::node::add_child(&self.api, self.rep, child.rep)
            .map_err(|e| e.to_string())
    }

    #[wasm_bindgen(js_name = "removeChild")]
    pub fn remove_child(&self, child: &Self) -> Result<(), String> {
        shared::wired::scene::node::remove_child(&self.api, self.rep, child.rep)
            .map_err(|e| e.to_string())
    }

    pub fn mesh(&self) -> Option<MeshHandle> {
        let rep = shared::wired::scene::node::mesh(&self.api, self.rep).ok()??;
        Some(MeshHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "setMesh")]
    pub fn set_mesh(&self, value: JsValue) -> Result<(), String> {
        shared::wired::scene::node::set_mesh(&self.api, self.rep, opt_rep(&value))
            .map_err(|e| e.to_string())
    }

    pub fn material(&self) -> Option<MaterialHandle> {
        let rep = shared::wired::scene::node::material(&self.api, self.rep).ok()??;
        Some(MaterialHandle::new(rep, Arc::clone(&self.api)))
    }

    #[wasm_bindgen(js_name = "setMaterial")]
    pub fn set_material(&self, value: JsValue) -> Result<(), String> {
        shared::wired::scene::node::set_material(&self.api, self.rep, opt_rep(&value))
            .map_err(|e| e.to_string())
    }

    pub async fn collider(&self) -> JsValue {
        shared::wired::scene::node::collider(&self.api, self.rep).await
            .ok()
            .flatten()
            .map_or(JsValue::NULL, collider_to_js)
    }

    #[wasm_bindgen(js_name = "setCollider")]
    pub async fn set_collider(&self, value: JsValue) -> Result<(), String> {
        shared::wired::scene::node::set_collider(&self.api, self.rep, js_to_collider(&value))
            .await
            .map_err(|e| e.to_string())
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
    pub fn set_rigid_body(&self, value: JsValue) -> Result<(), String> {
        let rb = value.as_string().and_then(|s| match s.as_str() {
            "dynamic" => Some(NodeRigidBody::Dynamic),
            "fixed" => Some(NodeRigidBody::Fixed),
            "kinematic" => Some(NodeRigidBody::Kinematic),
            _ => None,
        });
        shared::wired::scene::node::set_rigid_body(&self.api, self.rep, rb)
            .map_err(|e| e.to_string())
    }
}
