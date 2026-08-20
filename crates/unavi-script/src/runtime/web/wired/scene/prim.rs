use std::sync::Arc;

use hsd::attributes::image::{
    AddressMode,
    FilterMode,
    ImageAttr,
};
use unavi_util::async_task::spawn_async_task;
use wasm_bindgen::{
    JsValue,
    prelude::*,
};

use super::{
    shader_graph,
    util::{
        bytes32_to_js,
        js_to_bytes32,
        js_to_f32s,
        js_to_u32s,
        js_to_vec3,
        js_to_xform,
        obj_get,
        obj_get_bool,
        obj_get_f32,
        obj_get_string,
        obj_set,
        vec3_to_js,
        xform_to_js,
    },
};
use crate::runtime::{
    shared::{
        self,
        Api,
        wired::scene::prim::{
            PrimAlphaMode,
            PrimCollider,
            PrimColor,
            PrimGraphValue,
            PrimMaterial,
            PrimMesh,
            PrimPortal,
            PrimPortalDestination,
            PrimPortalReceptor,
            PrimRigidBody,
            PrimRigidBodyKind,
            PrimSpawn,
            PrimText,
            PrimTextAlign,
            PrimTextAnchor,
            PrimTextBillboard,
            PrimTopology,
        },
    },
    web::wired::{
        malformed,
        raise,
    },
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
            let api = Arc::clone(&self.api);
            let rep = self.rep;
            spawn_async_task(async move {
                let _ = shared::wired::scene::prim::on_drop(&api, rep).await;
            });
        }
    }
}

#[wasm_bindgen]
impl PrimHandle {
    #[wasm_bindgen(getter, js_name = "__rep")]
    // wasm_bindgen forbids `const fn`.
    #[expect(clippy::missing_const_for_fn)]
    pub fn js_rep(&self) -> u32 {
        self.rep
    }

    pub async fn id(&self) -> String {
        shared::wired::scene::prim::id(&self.api, self.rep)
            .await
            .unwrap_or_default()
    }

    #[wasm_bindgen(js_name = "clone")]
    pub async fn clone_prim(&self) -> Option<Self> {
        let rep = shared::wired::scene::prim::clone(&self.api, self.rep)
            .await
            .ok()?;
        Some(Self::new(rep, Arc::clone(&self.api)))
    }

    pub async fn parent(&self) -> Option<Self> {
        let rep = shared::wired::scene::prim::parent(&self.api, self.rep)
            .await
            .ok()??;
        Some(Self::new(rep, Arc::clone(&self.api)))
    }

    pub async fn children(&self) -> js_sys::Array {
        let Ok(reps) = shared::wired::scene::prim::children(&self.api, self.rep).await else {
            return js_sys::Array::new();
        };
        reps.into_iter()
            .map(|rep| JsValue::from(Self::new(rep, Arc::clone(&self.api))))
            .collect()
    }

    #[wasm_bindgen(js_name = "addChild")]
    pub async fn add_child(&self, child: &Self) -> Result<(), JsValue> {
        shared::wired::scene::prim::add_child(&self.api, self.rep, child.rep)
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "removeChild")]
    pub async fn remove_child(&self, child: &Self) -> Result<(), JsValue> {
        shared::wired::scene::prim::remove_child(&self.api, self.rep, child.rep)
            .await
            .map_err(raise)
    }

    pub async fn name(&self) -> Option<String> {
        shared::wired::scene::prim::name(&self.api, self.rep)
            .await
            .ok()
            .flatten()
    }

    #[wasm_bindgen(js_name = "setName")]
    pub async fn set_name(&self, value: Option<String>) -> Result<(), JsValue> {
        shared::wired::scene::prim::set_name(&self.api, self.rep, value)
            .await
            .map_err(raise)
    }

    pub async fn prefab(&self) -> JsValue {
        match shared::wired::scene::prim::prefab(&self.api, self.rep).await {
            Ok(Some(b)) => js_sys::Uint8Array::from(b.as_slice()).into(),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setPrefab")]
    pub async fn set_prefab(&self, value: JsValue) -> Result<(), JsValue> {
        shared::wired::scene::prim::set_prefab(&self.api, self.rep, js_to_bytes(&value))
            .await
            .map_err(raise)
    }

    pub async fn xform(&self) -> JsValue {
        match shared::wired::scene::prim::xform(&self.api, self.rep).await {
            Ok(Some(x)) => xform_to_js(&x),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setXform")]
    pub async fn set_xform(&self, value: JsValue) -> Result<(), JsValue> {
        let xf = js_to_xform(&value);
        shared::wired::scene::prim::set_xform(&self.api, self.rep, xf)
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "globalXform")]
    pub async fn global_xform(&self) -> JsValue {
        let x = shared::wired::scene::prim::global_xform(&self.api, self.rep)
            .await
            .unwrap_or_default();
        xform_to_js(&x)
    }

    #[wasm_bindgen(js_name = "gravityScale")]
    pub async fn gravity_scale(&self) -> f32 {
        shared::wired::scene::prim::gravity_scale(&self.api, self.rep)
            .await
            .unwrap_or(1.0)
    }

    #[wasm_bindgen(js_name = "setGravityScale")]
    pub async fn set_gravity_scale(&self, value: f32) -> Result<(), JsValue> {
        shared::wired::scene::prim::set_gravity_scale(&self.api, self.rep, value)
            .await
            .map_err(raise)
    }

    pub async fn mesh(&self) -> JsValue {
        match shared::wired::scene::prim::mesh(&self.api, self.rep).await {
            Ok(Some(m)) => mesh_to_js(m),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setMesh")]
    pub async fn set_mesh(&self, value: JsValue) -> Result<(), JsValue> {
        let m = js_to_mesh(&value);
        shared::wired::scene::prim::set_mesh(&self.api, self.rep, m)
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "setMeshStream")]
    pub async fn set_mesh_stream(&self, key: String, values: JsValue) -> Result<(), JsValue> {
        shared::wired::scene::prim::set_mesh_stream(&self.api, self.rep, key, js_to_f32s(values))
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "setMeshIndicesU32")]
    pub async fn set_mesh_indices_u32(&self, values: JsValue) -> Result<(), JsValue> {
        shared::wired::scene::prim::set_mesh_indices_u32(&self.api, self.rep, js_to_u32s(values))
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "meshStream")]
    pub async fn mesh_stream(&self, key: String) -> JsValue {
        match shared::wired::scene::prim::mesh_stream(&self.api, self.rep, key).await {
            Ok(Some(v)) => js_sys::Float32Array::from(v.as_slice()).into(),
            _ => JsValue::UNDEFINED,
        }
    }

    pub async fn material(&self) -> JsValue {
        match shared::wired::scene::prim::material(&self.api, self.rep).await {
            Ok(Some(m)) => material_to_js(&m),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setMaterial")]
    pub async fn set_material(&self, value: JsValue) -> Result<(), JsValue> {
        let m = js_to_material(&value);
        shared::wired::scene::prim::set_material(&self.api, self.rep, m)
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "setMaterialGraph")]
    pub async fn set_material_graph(&self, value: JsValue) -> Result<(), JsValue> {
        let graph = shader_graph::js_to_graph(&value).map_err(malformed)?;
        shared::wired::scene::prim::set_material_graph(&self.api, self.rep, graph)
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "graphOverrides")]
    pub async fn graph_overrides(&self) -> js_sys::Array {
        let Ok(items) = shared::wired::scene::prim::graph_overrides(&self.api, self.rep).await
        else {
            return js_sys::Array::new();
        };
        items
            .into_iter()
            .map(|(index, value)| {
                let tup = js_sys::Array::new();
                tup.push(&JsValue::from(index));
                tup.push(&shader_graph::graph_value_to_js(shader_graph::graph_value(
                    value,
                )));
                JsValue::from(tup)
            })
            .collect()
    }

    #[wasm_bindgen(js_name = "setGraphOverrides")]
    pub async fn set_graph_overrides(&self, values: JsValue) -> Result<(), JsValue> {
        let values = js_to_graph_overrides(&values).map_err(malformed)?;
        shared::wired::scene::prim::set_graph_overrides(&self.api, self.rep, values)
            .await
            .map_err(raise)
    }

    pub async fn image(&self) -> JsValue {
        match shared::wired::scene::prim::image(&self.api, self.rep).await {
            Ok(Some(img)) => image_to_js(&img),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setImage")]
    pub async fn set_image(&self, value: JsValue) -> Result<(), JsValue> {
        let img = js_to_image(&value);
        shared::wired::scene::prim::set_image(&self.api, self.rep, img)
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "setImageData")]
    pub async fn set_image_data(&self, bytes: JsValue) -> Result<(), JsValue> {
        shared::wired::scene::prim::set_image_data(&self.api, self.rep, js_to_bytes(&bytes))
            .await
            .map_err(raise)
    }

    pub async fn collider(&self) -> JsValue {
        match shared::wired::scene::prim::collider(&self.api, self.rep).await {
            Ok(Some(c)) => collider_to_js(c),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setCollider")]
    pub async fn set_collider(&self, value: JsValue) -> Result<(), JsValue> {
        let c = js_to_collider(&value);
        shared::wired::scene::prim::set_collider(&self.api, self.rep, c)
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "setColliderVertices")]
    pub async fn set_collider_vertices(&self, values: JsValue) -> Result<(), JsValue> {
        shared::wired::scene::prim::set_collider_vertices(&self.api, self.rep, js_to_f32s(values))
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "setColliderIndices")]
    pub async fn set_collider_indices(&self, values: JsValue) -> Result<(), JsValue> {
        shared::wired::scene::prim::set_collider_indices(&self.api, self.rep, js_to_u32s(values))
            .await
            .map_err(raise)
    }

    #[wasm_bindgen(js_name = "rigidBody")]
    pub async fn rigid_body(&self) -> JsValue {
        match shared::wired::scene::prim::rigid_body(&self.api, self.rep).await {
            Ok(Some(rb)) => rigid_body_to_js(&rb),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setRigidBody")]
    pub async fn set_rigid_body(&self, value: JsValue) -> Result<(), JsValue> {
        let rb = js_to_rigid_body(&value);
        shared::wired::scene::prim::set_rigid_body(&self.api, self.rep, rb)
            .await
            .map_err(raise)
    }

    pub async fn portal(&self) -> JsValue {
        match shared::wired::scene::prim::portal(&self.api, self.rep).await {
            Ok(Some(p)) => portal_to_js(&p),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setPortal")]
    pub async fn set_portal(&self, value: JsValue) -> Result<(), JsValue> {
        let value = js_to_portal(&value).map_err(malformed)?;
        shared::wired::scene::prim::set_portal(&self.api, self.rep, value)
            .await
            .map_err(raise)
    }

    pub async fn spawn(&self) -> JsValue {
        match shared::wired::scene::prim::spawn(&self.api, self.rep).await {
            Ok(Some(s)) => spawn_to_js(&s),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setSpawn")]
    pub async fn set_spawn(&self, value: JsValue) -> Result<(), JsValue> {
        let value = js_to_spawn(&value);
        shared::wired::scene::prim::set_spawn(&self.api, self.rep, value)
            .await
            .map_err(raise)
    }

    pub async fn text(&self) -> JsValue {
        match shared::wired::scene::prim::text(&self.api, self.rep).await {
            Ok(Some(t)) => text_to_js(&t),
            _ => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = "setText")]
    pub async fn set_text(&self, value: JsValue) -> Result<(), JsValue> {
        let value = js_to_text(&value);
        shared::wired::scene::prim::set_text(&self.api, self.rep, value)
            .await
            .map_err(raise)
    }

    pub async fn relationships(&self) -> js_sys::Array {
        let Ok(items) = shared::wired::scene::prim::relationships(&self.api, self.rep).await else {
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
    pub async fn get_relationship(&self, key: String) -> Option<String> {
        shared::wired::scene::prim::get_relationship(&self.api, self.rep, key)
            .await
            .ok()
            .flatten()
    }

    #[wasm_bindgen(js_name = "setRelationship")]
    pub async fn set_relationship(
        &self,
        key: String,
        target: Option<String>,
    ) -> Result<(), JsValue> {
        shared::wired::scene::prim::set_relationship(&self.api, self.rep, key, target)
            .await
            .map_err(raise)
    }
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
    obj.into()
}

fn js_to_mesh(v: &JsValue) -> Option<PrimMesh> {
    if v.is_null() || v.is_undefined() {
        return None;
    }
    Some(PrimMesh {
        topology: js_to_topology(&obj_get(v, "topology")),
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

fn text_to_js(t: &PrimText) -> JsValue {
    let obj = js_sys::Object::new();
    obj_set(&obj, "value", &JsValue::from_str(&t.value));
    if let Some(v) = t.size {
        obj_set(&obj, "size", &v.into());
    }
    if let Some(v) = t.align {
        obj_set(
            &obj,
            "align",
            &JsValue::from_str(match v {
                PrimTextAlign::Left => "left",
                PrimTextAlign::Center => "center",
                PrimTextAlign::Right => "right",
            }),
        );
    }
    if let Some(v) = t.anchor {
        obj_set(
            &obj,
            "anchor",
            &JsValue::from_str(match v {
                PrimTextAnchor::Baseline => "baseline",
                PrimTextAnchor::Top => "top",
                PrimTextAnchor::Middle => "middle",
                PrimTextAnchor::Bottom => "bottom",
            }),
        );
    }
    if let Some(v) = t.wrap {
        obj_set(&obj, "wrap", &v.into());
    }
    if let Some(v) = t.line_height {
        obj_set(&obj, "lineHeight", &v.into());
    }
    if let Some(v) = &t.color {
        obj_set(&obj, "color", &color_to_js(v));
    }
    if let Some(v) = &t.outline {
        obj_set(&obj, "outline", &color_to_js(v));
    }
    if let Some(v) = t.outline_width {
        obj_set(&obj, "outlineWidth", &v.into());
    }
    if let Some(v) = t.emissive {
        obj_set(&obj, "emissive", &v.into());
    }
    if let Some(v) = t.billboard {
        obj_set(
            &obj,
            "billboard",
            &JsValue::from_str(match v {
                PrimTextBillboard::None => "none",
                PrimTextBillboard::Yaw => "yaw",
                PrimTextBillboard::Full => "full",
            }),
        );
    }
    obj.into()
}

fn js_to_text(v: &JsValue) -> Option<PrimText> {
    if v.is_null() || v.is_undefined() {
        return None;
    }
    Some(PrimText {
        value:         obj_get_string(v, "value").unwrap_or_default(),
        size:          obj_get_f32(v, "size"),
        align:         obj_get_string(v, "align").and_then(|s| match s.as_str() {
            "left" => Some(PrimTextAlign::Left),
            "center" => Some(PrimTextAlign::Center),
            "right" => Some(PrimTextAlign::Right),
            _ => None,
        }),
        anchor:        obj_get_string(v, "anchor").and_then(|s| match s.as_str() {
            "baseline" => Some(PrimTextAnchor::Baseline),
            "top" => Some(PrimTextAnchor::Top),
            "middle" => Some(PrimTextAnchor::Middle),
            "bottom" => Some(PrimTextAnchor::Bottom),
            _ => None,
        }),
        wrap:          obj_get_f32(v, "wrap"),
        line_height:   obj_get_f32(v, "lineHeight"),
        color:         js_to_color(&obj_get(v, "color")),
        outline:       js_to_color(&obj_get(v, "outline")),
        outline_width: obj_get_f32(v, "outlineWidth"),
        emissive:      obj_get_f32(v, "emissive"),
        billboard:     obj_get_string(v, "billboard").and_then(|s| match s.as_str() {
            "none" => Some(PrimTextBillboard::None),
            "yaw" => Some(PrimTextBillboard::Yaw),
            "full" => Some(PrimTextBillboard::Full),
            _ => None,
        }),
    })
}

fn material_to_js(m: &PrimMaterial) -> JsValue {
    let obj = js_sys::Object::new();
    if let Some(v) = m.alpha_cutoff {
        obj_set(&obj, "alphaCutoff", &v.into());
    }
    if let Some(v) = m.alpha_mode {
        obj_set(&obj, "alphaMode", &alpha_mode_to_js(v));
    }
    if let Some(v) = &m.base_color {
        obj_set(&obj, "baseColor", &color_to_js(v));
    }
    if let Some(v) = m.double_sided {
        obj_set(&obj, "doubleSided", &v.into());
    }
    if let Some(v) = &m.emissive {
        obj_set(&obj, "emissive", &color_to_js(v));
    }
    if let Some(v) = m.metallic {
        obj_set(&obj, "metallic", &v.into());
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
        alpha_cutoff: obj_get_f32(v, "alphaCutoff"),
        alpha_mode:   js_to_alpha_mode(&obj_get(v, "alphaMode")),
        base_color:   js_to_color(&obj_get(v, "baseColor")),
        double_sided: obj_get_bool(v, "doubleSided"),
        emissive:     js_to_color(&obj_get(v, "emissive")),
        metallic:     obj_get_f32(v, "metallic"),
        roughness:    obj_get_f32(v, "roughness"),
    })
}

fn address_mode_to_js(mode: AddressMode) -> JsValue {
    JsValue::from_str(match mode {
        AddressMode::Repeat => "repeat",
        AddressMode::MirrorRepeat => "mirror-repeat",
        AddressMode::ClampToEdge => "clamp-to-edge",
    })
}

fn js_to_address_mode(v: &JsValue) -> Option<AddressMode> {
    Some(match v.as_string()?.as_str() {
        "repeat" => AddressMode::Repeat,
        "mirror-repeat" => AddressMode::MirrorRepeat,
        "clamp-to-edge" => AddressMode::ClampToEdge,
        _ => return None,
    })
}

fn filter_mode_to_js(mode: FilterMode) -> JsValue {
    JsValue::from_str(match mode {
        FilterMode::Linear => "linear",
        FilterMode::Nearest => "nearest",
    })
}

fn js_to_filter_mode(v: &JsValue) -> Option<FilterMode> {
    Some(match v.as_string()?.as_str() {
        "linear" => FilterMode::Linear,
        "nearest" => FilterMode::Nearest,
        _ => return None,
    })
}

fn image_to_js(img: &ImageAttr) -> JsValue {
    let obj = js_sys::Object::new();
    for (key, mode) in [
        ("addressModeU", img.address_mode_u),
        ("addressModeV", img.address_mode_v),
        ("addressModeW", img.address_mode_w),
    ] {
        if let Some(mode) = mode {
            obj_set(&obj, key, &address_mode_to_js(mode));
        }
    }
    for (key, mode) in [
        ("magFilter", img.mag_filter),
        ("minFilter", img.min_filter),
        ("mipmapFilter", img.mipmap_filter),
    ] {
        if let Some(mode) = mode {
            obj_set(&obj, key, &filter_mode_to_js(mode));
        }
    }
    if let Some(v) = img.srgb {
        obj_set(&obj, "srgb", &v.into());
    }
    obj.into()
}

fn js_to_image(v: &JsValue) -> Option<ImageAttr> {
    if v.is_null() || v.is_undefined() {
        return None;
    }
    Some(ImageAttr {
        address_mode_u: js_to_address_mode(&obj_get(v, "addressModeU")),
        address_mode_v: js_to_address_mode(&obj_get(v, "addressModeV")),
        address_mode_w: js_to_address_mode(&obj_get(v, "addressModeW")),
        mag_filter:     js_to_filter_mode(&obj_get(v, "magFilter")),
        min_filter:     js_to_filter_mode(&obj_get(v, "minFilter")),
        mipmap_filter:  js_to_filter_mode(&obj_get(v, "mipmapFilter")),
        srgb:           obj_get_bool(v, "srgb"),
    })
}

fn variant(tag: &str, val: JsValue) -> JsValue {
    let obj = js_sys::Object::new();
    obj_set(&obj, "tag", &tag.into());
    obj_set(&obj, "val", &val);
    obj.into()
}

/// A variant case carrying nothing has no `val` at all, rather than an
/// undefined one.
fn unit_variant(tag: &str) -> JsValue {
    let obj = js_sys::Object::new();
    obj_set(&obj, "tag", &tag.into());
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
        PrimCollider::ConvexHull => unit_variant("convex-hull"),
        PrimCollider::Cuboid([x, y, z]) => variant("cuboid", vec3_to_js(x, y, z)),
        PrimCollider::Cylinder { height, radius } => {
            variant("cylinder", record2("height", height, "radius", radius))
        }
        PrimCollider::Sphere(r) => variant("sphere", r.into()),
        PrimCollider::Trimesh => unit_variant("trimesh"),
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
        "convex-hull" => PrimCollider::ConvexHull,
        "cuboid" => PrimCollider::Cuboid(js_to_vec3(&val, [0.0; 3])),
        "cylinder" => PrimCollider::Cylinder {
            height: obj_get_f32(&val, "height").unwrap_or(0.0),
            radius: obj_get_f32(&val, "radius").unwrap_or(0.0),
        },
        "sphere" => PrimCollider::Sphere(val.as_f64().unwrap_or(0.0) as f32),
        "trimesh" => PrimCollider::Trimesh,
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
        obj_set(&obj, "angularDamping", &v.into());
    }
    if let Some(v) = rb.friction {
        obj_set(&obj, "friction", &v.into());
    }
    if let Some(v) = rb.linear_damping {
        obj_set(&obj, "linearDamping", &v.into());
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
        kind:            js_to_rigid_kind(&obj_get(v, "kind")),
        angular_damping: obj_get_f32(v, "angularDamping"),
        friction:        obj_get_f32(v, "friction"),
        linear_damping:  obj_get_f32(v, "linearDamping"),
        mass:            obj_get_f32(v, "mass"),
        restitution:     obj_get_f32(v, "restitution"),
    })
}

fn portal_to_js(p: &PrimPortal) -> JsValue {
    let obj = js_sys::Object::new();
    if let Some(d) = &p.destination {
        let dest = js_sys::Object::new();
        if let Some(r) = &d.receptor {
            let rec = js_sys::Object::new();
            obj_set(&rec, "document", &bytes32_to_js(&r.document));
            obj_set(&rec, "prim", &JsValue::from_str(&r.prim));
            obj_set(&dest, "receptor", &rec.into());
        }
        obj_set(&dest, "space", &bytes32_to_js(&d.space));
        obj_set(&obj, "destination", &dest.into());
    }
    obj_set(&obj, "sizeX", &p.size_x.into());
    obj_set(&obj, "sizeY", &p.size_y.into());
    obj.into()
}

fn spawn_to_js(s: &PrimSpawn) -> JsValue {
    let obj = js_sys::Object::new();
    obj_set(&obj, "radius", &s.radius.into());
    obj.into()
}

fn js_to_spawn(v: &JsValue) -> Option<PrimSpawn> {
    if v.is_null() || v.is_undefined() {
        return None;
    }
    Some(PrimSpawn {
        radius: obj_get_f32(v, "radius").unwrap_or(0.0),
    })
}

fn js_to_portal(v: &JsValue) -> Result<Option<PrimPortal>, String> {
    if v.is_null() || v.is_undefined() {
        return Ok(None);
    }
    let destination = {
        let d = obj_get(v, "destination");
        if d.is_null() || d.is_undefined() {
            None
        } else {
            let receptor = {
                let r = obj_get(&d, "receptor");
                if r.is_null() || r.is_undefined() {
                    None
                } else {
                    let document = js_to_bytes32(&obj_get(&r, "document"))
                        .ok_or_else(|| "portal receptor document must be 32 bytes".to_string())?;
                    let prim = obj_get_string(&r, "prim").unwrap_or_default();
                    Some(PrimPortalReceptor { document, prim })
                }
            };
            let space = js_to_bytes32(&obj_get(&d, "space"))
                .ok_or_else(|| "portal destination space must be 32 bytes".to_string())?;
            Some(PrimPortalDestination { receptor, space })
        }
    };
    Ok(Some(PrimPortal {
        destination,
        size_x: obj_get_f32(v, "sizeX").unwrap_or(0.0),
        size_y: obj_get_f32(v, "sizeY").unwrap_or(0.0),
    }))
}

fn js_to_bytes(v: &JsValue) -> Option<Vec<u8>> {
    if v.is_null() || v.is_undefined() {
        return None;
    }
    Some(js_sys::Uint8Array::new(v).to_vec())
}

fn js_to_graph_overrides(v: &JsValue) -> Result<Vec<(u16, PrimGraphValue)>, String> {
    if v.is_null() || v.is_undefined() {
        return Ok(Vec::new());
    }
    js_sys::Array::from(v)
        .iter()
        .map(|entry| {
            let tup = js_sys::Array::from(&entry);
            let index = tup
                .get(0)
                .as_f64()
                .and_then(|i| u16::try_from(i as i64).ok())
                .ok_or_else(|| "a graph override index is a u16".to_string())?;
            let value = shader_graph::js_to_graph_value(&tup.get(1))?;
            Ok((index, shader_graph::prim_value(value)))
        })
        .collect()
}
