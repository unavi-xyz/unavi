use std::sync::Arc;

use wasm_bindgen::{JsValue, prelude::*};

use crate::runtime::shared::{
    self, Api,
    wired::scene::prim::{
        PrimAlphaMode, PrimCollider, PrimColor, PrimImage, PrimMaterial, PrimMesh, PrimRigidBody,
        PrimRigidBodyKind, PrimTopology, PrimXform,
    },
};

use super::util::{
    bytes32_to_js, js_to_bytes32, js_to_f32s, js_to_quat, js_to_u32s, js_to_vec3, obj_get,
    obj_get_bool, obj_get_f32, obj_get_i32, obj_get_string, obj_set, quat_to_js, vec3_to_js,
};

#[wasm_bindgen]
pub struct PrimHandle {
    rep: u32,
    api: Arc<Api>,
}

impl PrimHandle {
    pub const fn new(rep: u32, api: Arc<Api>) -> Self {
        Self { rep, api }
    }

    pub const fn rep(&self) -> u32 {
        self.rep
    }
}

impl Drop for PrimHandle {
    fn drop(&mut self) {
        if self.rep != u32::MAX {
            let _ = shared::wired::scene::prim::on_drop(&self.api, self.rep);
        }
    }
}

#[wasm_bindgen]
impl PrimHandle {
    #[wasm_bindgen(getter, js_name = "__rep")]
    pub fn js_rep(&self) -> u32 {
        self.rep
    }

    pub fn id(&self) -> String {
        shared::wired::scene::prim::id(&self.api, self.rep).unwrap_or_default()
    }

    #[wasm_bindgen(js_name = "clone")]
    pub fn clone_prim(&self) -> Option<Self> {
        let rep = shared::wired::scene::prim::clone(&self.api, self.rep).ok()?;
        Some(Self::new(rep, Arc::clone(&self.api)))
    }

    pub fn parent(&self) -> Option<PrimHandle> {
        let rep = shared::wired::scene::prim::parent(&self.api, self.rep).ok()??;
        Some(PrimHandle::new(rep, Arc::clone(&self.api)))
    }

    pub fn children(&self) -> js_sys::Array {
        let Ok(reps) = shared::wired::scene::prim::children(&self.api, self.rep) else {
            return js_sys::Array::new();
        };
        reps.into_iter()
            .map(|rep| JsValue::from(PrimHandle::new(rep, Arc::clone(&self.api))))
            .collect()
    }

    #[wasm_bindgen(js_name = "addChild")]
    pub fn add_child(&self, child: &PrimHandle) -> Result<(), String> {
        shared::wired::scene::prim::add_child(&self.api, self.rep, child.rep)
            .map_err(|e| e.to_string())
    }

    #[wasm_bindgen(js_name = "removeChild")]
    pub fn remove_child(&self, child: &PrimHandle) -> Result<(), String> {
        shared::wired::scene::prim::remove_child(&self.api, self.rep, child.rep)
            .map_err(|e| e.to_string())
    }

    pub fn name(&self) -> Option<String> {
        shared::wired::scene::prim::name(&self.api, self.rep)
            .ok()
            .flatten()
    }

    #[wasm_bindgen(js_name = "setName")]
    pub fn set_name(&self, value: Option<String>) -> Result<(), String> {
        shared::wired::scene::prim::set_name(&self.api, self.rep, value).map_err(|e| e.to_string())
    }

    pub fn asset(&self) -> JsValue {
        match shared::wired::scene::prim::asset(&self.api, self.rep) {
            Ok(Some(b)) => js_sys::Uint8Array::from(b.as_slice()).into(),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setAsset")]
    pub fn set_asset(&self, value: JsValue) -> Result<(), String> {
        let bytes = if value.is_null() || value.is_undefined() {
            None
        } else {
            Some(js_sys::Uint8Array::new(&value).to_vec())
        };
        shared::wired::scene::prim::set_asset(&self.api, self.rep, bytes).map_err(|e| e.to_string())
    }

    pub fn xform(&self) -> JsValue {
        match shared::wired::scene::prim::xform(&self.api, self.rep) {
            Ok(Some(x)) => xform_to_js(&x),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setXform")]
    pub fn set_xform(&self, value: JsValue) -> Result<(), String> {
        let xf = js_to_xform(&value);
        shared::wired::scene::prim::set_xform(&self.api, self.rep, xf).map_err(|e| e.to_string())
    }

    #[wasm_bindgen(js_name = "globalXform")]
    pub fn global_xform(&self) -> JsValue {
        let x = shared::wired::scene::prim::global_xform(&self.api, self.rep).unwrap_or_default();
        xform_to_js(&x)
    }

    pub fn mesh(&self) -> JsValue {
        match shared::wired::scene::prim::mesh(&self.api, self.rep) {
            Ok(Some(m)) => mesh_to_js(m),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setMesh")]
    pub fn set_mesh(&self, value: JsValue) -> Result<(), String> {
        let m = js_to_mesh(&value);
        shared::wired::scene::prim::set_mesh(&self.api, self.rep, m).map_err(|e| e.to_string())
    }

    #[wasm_bindgen(js_name = "setMeshStream")]
    pub async fn set_mesh_stream(&self, key: String, values: JsValue) -> Result<(), String> {
        shared::wired::scene::prim::set_mesh_stream(&self.api, self.rep, key, js_to_f32s(values))
            .await
            .map_err(|e| e.to_string())
    }

    #[wasm_bindgen(js_name = "setMeshIndicesU32")]
    pub async fn set_mesh_indices_u32(&self, values: JsValue) -> Result<(), String> {
        shared::wired::scene::prim::set_mesh_indices_u32(&self.api, self.rep, js_to_u32s(values))
            .await
            .map_err(|e| e.to_string())
    }

    pub fn material(&self) -> JsValue {
        match shared::wired::scene::prim::material(&self.api, self.rep) {
            Ok(Some(m)) => material_to_js(&m),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setMaterial")]
    pub fn set_material(&self, value: JsValue) -> Result<(), String> {
        let m = js_to_material(&value);
        shared::wired::scene::prim::set_material(&self.api, self.rep, m).map_err(|e| e.to_string())
    }

    pub fn image(&self) -> JsValue {
        match shared::wired::scene::prim::image(&self.api, self.rep) {
            Ok(Some(img)) => image_to_js(&img),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setImage")]
    pub fn set_image(&self, value: JsValue) -> Result<(), String> {
        let img = js_to_image(&value);
        shared::wired::scene::prim::set_image(&self.api, self.rep, img).map_err(|e| e.to_string())
    }

    pub fn collider(&self) -> JsValue {
        match shared::wired::scene::prim::collider(&self.api, self.rep) {
            Ok(Some(c)) => collider_to_js(c),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setCollider")]
    pub fn set_collider(&self, value: JsValue) -> Result<(), String> {
        let c = js_to_collider(&value);
        shared::wired::scene::prim::set_collider(&self.api, self.rep, c).map_err(|e| e.to_string())
    }

    #[wasm_bindgen(js_name = "rigidBody")]
    pub fn rigid_body(&self) -> JsValue {
        match shared::wired::scene::prim::rigid_body(&self.api, self.rep) {
            Ok(Some(rb)) => rigid_body_to_js(&rb),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setRigidBody")]
    pub fn set_rigid_body(&self, value: JsValue) -> Result<(), String> {
        let rb = js_to_rigid_body(&value);
        shared::wired::scene::prim::set_rigid_body(&self.api, self.rep, rb)
            .map_err(|e| e.to_string())
    }

    pub fn relationships(&self) -> js_sys::Array {
        let Ok(items) = shared::wired::scene::prim::relationships(&self.api, self.rep) else {
            return js_sys::Array::new();
        };
        items
            .into_iter()
            .map(|(k, v)| {
                let tup = js_sys::Array::new();
                tup.push(&JsValue::from_str(&k));
                tup.push(&JsValue::from_str(&v));
                JsValue::from(tup)
            })
            .collect()
    }

    #[wasm_bindgen(js_name = "getRelationship")]
    pub fn get_relationship(&self, key: String) -> Option<String> {
        shared::wired::scene::prim::get_relationship(&self.api, self.rep, key)
            .ok()
            .flatten()
    }

    #[wasm_bindgen(js_name = "setRelationship")]
    pub fn set_relationship(&self, key: String, target: Option<String>) -> Result<(), String> {
        shared::wired::scene::prim::set_relationship(&self.api, self.rep, key, target)
            .map_err(|e| e.to_string())
    }
}

fn xform_to_js(x: &PrimXform) -> JsValue {
    let obj = js_sys::Object::new();
    obj_set(
        &obj,
        "translation",
        &vec3_to_js(x.translation[0], x.translation[1], x.translation[2]),
    );
    obj_set(
        &obj,
        "rotation",
        &quat_to_js(x.rotation[0], x.rotation[1], x.rotation[2], x.rotation[3]),
    );
    obj_set(
        &obj,
        "scale",
        &vec3_to_js(x.scale[0], x.scale[1], x.scale[2]),
    );
    obj.into()
}

fn js_to_xform(v: &JsValue) -> Option<PrimXform> {
    if v.is_null() || v.is_undefined() {
        return None;
    }
    Some(PrimXform {
        translation: js_to_vec3(&obj_get(v, "translation"), [0.0; 3]),
        rotation: js_to_quat(&obj_get(v, "rotation"), [0.0, 0.0, 0.0, 1.0]),
        scale: js_to_vec3(&obj_get(v, "scale"), [1.0; 3]),
    })
}

fn topology_to_js(t: PrimTopology) -> JsValue {
    JsValue::from_str(match t {
        PrimTopology::PointList => "point-list",
        PrimTopology::LineList => "line-list",
        PrimTopology::LineStrip => "line-strip",
        PrimTopology::TriangleList => "triangle-list",
        PrimTopology::TriangleStrip => "triangle-strip",
    })
}

fn js_to_topology(v: &JsValue) -> PrimTopology {
    match v.as_string().as_deref() {
        Some("point-list") => PrimTopology::PointList,
        Some("line-list") => PrimTopology::LineList,
        Some("line-strip") => PrimTopology::LineStrip,
        Some("triangle-strip") => PrimTopology::TriangleStrip,
        _ => PrimTopology::TriangleList,
    }
}

fn mesh_to_js(m: PrimMesh) -> JsValue {
    let obj = js_sys::Object::new();
    obj_set(&obj, "topology", &topology_to_js(m.topology));
    let attrs = js_sys::Array::new();
    for (k, v) in m.attributes {
        let tup = js_sys::Array::new();
        tup.push(&JsValue::from_str(&k));
        tup.push(&bytes32_to_js(&v));
        attrs.push(&tup);
    }
    obj_set(&obj, "attributes", &attrs.into());
    obj_set(
        &obj,
        "indices",
        &m.indices.map_or(JsValue::UNDEFINED, |b| bytes32_to_js(&b)),
    );
    obj.into()
}

fn js_to_mesh(v: &JsValue) -> Option<PrimMesh> {
    if v.is_null() || v.is_undefined() {
        return None;
    }
    let topology = js_to_topology(&obj_get(v, "topology"));
    let attrs_val = obj_get(v, "attributes");
    let mut attributes = Vec::new();
    if attrs_val.is_object() {
        let arr = js_sys::Array::from(&attrs_val);
        for entry in arr.iter() {
            let tup = js_sys::Array::from(&entry);
            let Some(k) = tup.get(0).as_string() else {
                continue;
            };
            let Some(b) = js_to_bytes32(&tup.get(1)) else {
                continue;
            };
            attributes.push((k, b));
        }
    }
    let indices = js_to_bytes32(&obj_get(v, "indices"));
    Some(PrimMesh {
        topology,
        attributes,
        indices,
    })
}

fn alpha_mode_to_js(m: PrimAlphaMode) -> JsValue {
    JsValue::from_str(match m {
        PrimAlphaMode::Add => "add",
        PrimAlphaMode::Blend => "blend",
        PrimAlphaMode::Mask => "mask",
        PrimAlphaMode::Multiply => "multiply",
        PrimAlphaMode::Opaque => "opaque",
        PrimAlphaMode::PreMultiplied => "pre-multiplied",
    })
}

fn js_to_alpha_mode(v: &JsValue) -> Option<PrimAlphaMode> {
    Some(match v.as_string()?.as_str() {
        "add" => PrimAlphaMode::Add,
        "blend" => PrimAlphaMode::Blend,
        "mask" => PrimAlphaMode::Mask,
        "multiply" => PrimAlphaMode::Multiply,
        "opaque" => PrimAlphaMode::Opaque,
        "pre-multiplied" => PrimAlphaMode::PreMultiplied,
        _ => return None,
    })
}

fn color_to_js(c: &PrimColor) -> JsValue {
    let obj = js_sys::Object::new();
    obj_set(&obj, "r", &c.r.into());
    obj_set(&obj, "g", &c.g.into());
    obj_set(&obj, "b", &c.b.into());
    obj_set(&obj, "a", &c.a.into());
    obj.into()
}

fn js_to_color(v: &JsValue) -> Option<PrimColor> {
    if v.is_null() || v.is_undefined() {
        return None;
    }
    Some(PrimColor {
        r: obj_get_f32(v, "r").unwrap_or(1.0),
        g: obj_get_f32(v, "g").unwrap_or(1.0),
        b: obj_get_f32(v, "b").unwrap_or(1.0),
        a: obj_get_f32(v, "a").unwrap_or(1.0),
    })
}

fn material_to_js(m: &PrimMaterial) -> JsValue {
    let obj = js_sys::Object::new();
    if let Some(v) = m.alpha_cutoff {
        obj_set(&obj, "alpha-cutoff", &v.into());
    }
    if let Some(v) = m.alpha_mode {
        obj_set(&obj, "alpha-mode", &alpha_mode_to_js(v));
    }
    if let Some(v) = &m.base_color {
        obj_set(&obj, "base-color", &color_to_js(v));
    }
    if let Some(v) = &m.base_color_texture {
        obj_set(&obj, "base-color-texture", &JsValue::from_str(v));
    }
    if let Some(v) = m.double_sided {
        obj_set(&obj, "double-sided", &v.into());
    }
    if let Some(v) = &m.emissive {
        obj_set(&obj, "emissive", &color_to_js(v));
    }
    if let Some(v) = &m.emissive_texture {
        obj_set(&obj, "emissive-texture", &JsValue::from_str(v));
    }
    if let Some(v) = m.metallic {
        obj_set(&obj, "metallic", &v.into());
    }
    if let Some(v) = &m.metallic_roughness_texture {
        obj_set(&obj, "metallic-roughness-texture", &JsValue::from_str(v));
    }
    if let Some(v) = &m.normal_texture {
        obj_set(&obj, "normal-texture", &JsValue::from_str(v));
    }
    if let Some(v) = &m.occlusion_texture {
        obj_set(&obj, "occlusion-texture", &JsValue::from_str(v));
    }
    if let Some(v) = m.roughness {
        obj_set(&obj, "roughness", &v.into());
    }
    obj.into()
}

fn js_to_material(v: &JsValue) -> Option<PrimMaterial> {
    if v.is_null() || v.is_undefined() {
        return None;
    }
    Some(PrimMaterial {
        alpha_cutoff: obj_get_f32(v, "alpha-cutoff"),
        alpha_mode: js_to_alpha_mode(&obj_get(v, "alpha-mode")),
        base_color: js_to_color(&obj_get(v, "base-color")),
        base_color_texture: obj_get_string(v, "base-color-texture"),
        double_sided: obj_get_bool(v, "double-sided"),
        emissive: js_to_color(&obj_get(v, "emissive")),
        emissive_texture: obj_get_string(v, "emissive-texture"),
        metallic: obj_get_f32(v, "metallic"),
        metallic_roughness_texture: obj_get_string(v, "metallic-roughness-texture"),
        normal_texture: obj_get_string(v, "normal-texture"),
        occlusion_texture: obj_get_string(v, "occlusion-texture"),
        roughness: obj_get_f32(v, "roughness"),
    })
}

fn image_to_js(img: &PrimImage) -> JsValue {
    let obj = js_sys::Object::new();
    obj_set(&obj, "data", &bytes32_to_js(&img.data));
    if let Some(v) = img.address_mode_u {
        obj_set(&obj, "address-mode-u", &v.into());
    }
    if let Some(v) = img.address_mode_v {
        obj_set(&obj, "address-mode-v", &v.into());
    }
    if let Some(v) = img.address_mode_w {
        obj_set(&obj, "address-mode-w", &v.into());
    }
    if let Some(v) = img.mag_filter {
        obj_set(&obj, "mag-filter", &v.into());
    }
    if let Some(v) = img.min_filter {
        obj_set(&obj, "min-filter", &v.into());
    }
    if let Some(v) = img.mipmap_filter {
        obj_set(&obj, "mipmap-filter", &v.into());
    }
    if let Some(v) = img.srgb {
        obj_set(&obj, "srgb", &v.into());
    }
    obj.into()
}

fn js_to_image(v: &JsValue) -> Option<PrimImage> {
    if v.is_null() || v.is_undefined() {
        return None;
    }
    Some(PrimImage {
        data: js_to_bytes32(&obj_get(v, "data"))?,
        address_mode_u: obj_get_i32(v, "address-mode-u"),
        address_mode_v: obj_get_i32(v, "address-mode-v"),
        address_mode_w: obj_get_i32(v, "address-mode-w"),
        mag_filter: obj_get_i32(v, "mag-filter"),
        min_filter: obj_get_i32(v, "min-filter"),
        mipmap_filter: obj_get_i32(v, "mipmap-filter"),
        srgb: obj_get_bool(v, "srgb"),
    })
}

fn variant(tag: &str, val: JsValue) -> JsValue {
    let obj = js_sys::Object::new();
    obj_set(&obj, "tag", &tag.into());
    obj_set(&obj, "val", &val);
    obj.into()
}

fn collider_to_js(c: PrimCollider) -> JsValue {
    let record2 = |k1: &str, v1: f32, k2: &str, v2: f32| -> JsValue {
        let obj = js_sys::Object::new();
        obj_set(&obj, k1, &v1.into());
        obj_set(&obj, k2, &v2.into());
        obj.into()
    };
    match c {
        PrimCollider::Capsule { height, radius } => {
            variant("capsule", record2("height", height, "radius", radius))
        }
        PrimCollider::ConvexHull(hash) => variant("convex-hull", bytes32_to_js(&hash)),
        PrimCollider::Cuboid([x, y, z]) => variant("cuboid", vec3_to_js(x, y, z)),
        PrimCollider::Cylinder { height, radius } => {
            variant("cylinder", record2("height", height, "radius", radius))
        }
        PrimCollider::Sphere(r) => variant("sphere", r.into()),
        PrimCollider::Trimesh { indices, vertices } => {
            let val = js_sys::Object::new();
            obj_set(&val, "indices", &bytes32_to_js(&indices));
            obj_set(&val, "vertices", &bytes32_to_js(&vertices));
            variant("trimesh", val.into())
        }
    }
}

fn js_to_collider(value: &JsValue) -> Option<PrimCollider> {
    if value.is_null() || value.is_undefined() {
        return None;
    }
    let tag = obj_get_string(value, "tag")?;
    let val = obj_get(value, "val");
    Some(match tag.as_str() {
        "capsule" => PrimCollider::Capsule {
            height: obj_get_f32(&val, "height").unwrap_or(0.0),
            radius: obj_get_f32(&val, "radius").unwrap_or(0.0),
        },
        "convex-hull" => PrimCollider::ConvexHull(js_to_bytes32(&val)?),
        "cuboid" => PrimCollider::Cuboid(js_to_vec3(&val, [0.0; 3])),
        "cylinder" => PrimCollider::Cylinder {
            height: obj_get_f32(&val, "height").unwrap_or(0.0),
            radius: obj_get_f32(&val, "radius").unwrap_or(0.0),
        },
        "sphere" => PrimCollider::Sphere(val.as_f64().unwrap_or(0.0) as f32),
        "trimesh" => PrimCollider::Trimesh {
            indices: js_to_bytes32(&obj_get(&val, "indices"))?,
            vertices: js_to_bytes32(&obj_get(&val, "vertices"))?,
        },
        _ => return None,
    })
}

fn rigid_kind_to_js(k: PrimRigidBodyKind) -> JsValue {
    JsValue::from_str(match k {
        PrimRigidBodyKind::Dynamic => "dynamic",
        PrimRigidBodyKind::Kinematic => "kinematic",
        PrimRigidBodyKind::Static => "static",
    })
}

fn js_to_rigid_kind(v: &JsValue) -> PrimRigidBodyKind {
    match v.as_string().as_deref() {
        Some("kinematic") => PrimRigidBodyKind::Kinematic,
        Some("static") => PrimRigidBodyKind::Static,
        _ => PrimRigidBodyKind::Dynamic,
    }
}

fn rigid_body_to_js(rb: &PrimRigidBody) -> JsValue {
    let obj = js_sys::Object::new();
    obj_set(&obj, "kind", &rigid_kind_to_js(rb.kind));
    if let Some(v) = rb.angular_damping {
        obj_set(&obj, "angular-damping", &v.into());
    }
    if let Some(v) = rb.friction {
        obj_set(&obj, "friction", &v.into());
    }
    if let Some(v) = rb.linear_damping {
        obj_set(&obj, "linear-damping", &v.into());
    }
    if let Some(v) = rb.mass {
        obj_set(&obj, "mass", &v.into());
    }
    if let Some(v) = rb.restitution {
        obj_set(&obj, "restitution", &v.into());
    }
    obj.into()
}

fn js_to_rigid_body(v: &JsValue) -> Option<PrimRigidBody> {
    if v.is_null() || v.is_undefined() {
        return None;
    }
    Some(PrimRigidBody {
        kind: js_to_rigid_kind(&obj_get(v, "kind")),
        angular_damping: obj_get_f32(v, "angular-damping"),
        friction: obj_get_f32(v, "friction"),
        linear_damping: obj_get_f32(v, "linear-damping"),
        mass: obj_get_f32(v, "mass"),
        restitution: obj_get_f32(v, "restitution"),
    })
}
