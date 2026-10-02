//! Conversions between `jco`-lowered JS values and host types.
//!
//! Most shapes round-trip through `serde` and [`serde_wasm_bindgen`], mirroring
//! how jco lowers a record (camelCase fields), an enum (kebab-case string) or
//! a variant (`{tag, val}`, kebab-case tag, no `val` for an empty case). The
//! two exceptions are ids, which carry `u64` words as JS `bigint`s, and the
//! handful of `list<f32>`/`list<u32>`/`list<u8>` fields, which jco lowers as
//! typed arrays rather than plain JS arrays; `serde` has no vocabulary for
//! either, so both are built and read by hand, as [`id`] and the numeric-list
//! arms of [`property`]/[`wit_property`] do.
//!
//! A malformed guest value becomes [`ScriptError::invalid`], which a fallible
//! call lowers with [`raise`] and an infallible one lowers with [`trap`] —
//! never a silent default.

use std::fmt::Display;

use hsd::{
    attributes::{
        collider::ColliderKind,
        image::{
            AddressMode,
            FilterMode,
            ImageSampler,
        },
        material::{
            AlphaMode,
            ColorVec,
            MaterialAttr,
        },
        mesh::Topology,
        portal::{
            LinkId,
            PortalAttr,
            PortalDestination,
        },
        rigid_body::{
            RigidBodyAttr,
            RigidBodyKind,
        },
        spawn::SpawnAttr,
        text::{
            TextAlign,
            TextAnchor,
            TextAttr,
            TextBillboard,
        },
        xform::XformAttr,
    },
    id::{
        DocId,
        PrimId,
    },
};
use js_sys::{
    Array,
    Float32Array,
    Reflect,
    Uint32Array,
};
use serde::{
    Deserialize,
    Serialize,
    de::DeserializeOwned,
};
use unavi_policy::{
    permissions::HostApi,
    quota::{
        Flow,
        Stock,
    },
};
use wasm_bindgen::{
    JsError,
    JsValue,
};

use crate::{
    error::ScriptError,
    host::scene::{
        edit::{
            Edit,
            Layer,
        },
        place::Anchor,
        property::{
            Property,
            PropertyKey,
            Relation,
            VertexAttribute,
            custom_name,
        },
    },
};

/// Lowers `err` as a fallible call's `result<_, error>` rejection: jco catches
/// whatever a host import throws and reads it as the `err` case, so the
/// thrown value must already be the lowered variant. A handle the guest does
/// not hold has no such shape; it throws a real `Error` instead, which jco
/// cannot parse as a variant and so aborts the call, matching native's trap.
pub fn raise(err: ScriptError) -> JsValue {
    if err == ScriptError::InvalidHandle {
        return trap(&err).into();
    }
    to_js(&wire_error(err))
}

/// Lowers `err` for a call the WIT declares infallible: thrown as a plain
/// `Error`, which jco does not catch, aborting the guest the way a native
/// trap does.
pub fn trap(err: &impl Display) -> JsError {
    JsError::new(&err.to_string())
}

/// [`trap`], as the error arm of a `Result<T, JsValue>` returned from a call
/// the WIT declares infallible. The only error these calls can still carry is
/// [`ScriptError::InvalidHandle`] — a handle check, not a WIT-visible
/// failure — so every one of them traps rather than lowering an `error`.
pub fn into_trap(err: ScriptError) -> JsValue {
    trap(&err).into()
}

#[derive(Serialize)]
#[serde(tag = "tag", content = "val", rename_all = "kebab-case")]
enum WireError {
    InvalidArgument(String),
    NotFound,
    NotReady,
    Permission(WirePermission),
    Forbidden,
    RateLimited(WireRate),
    LimitReached(WireLimit),
    Internal(String),
}

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum WirePermission {
    Scene,
    CreateDocument,
    Commit,
    Event,
    Input,
    InputDevice,
    LocalAgent,
    Identity,
    Peer,
    Physics,
    Portal,
    Travel,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
enum WireRate {
    CreateDocument,
    CreatePrim,
    Emit,
    Upload,
    Portal,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
enum WireLimit {
    Documents,
    Prims,
    Handles,
    Subscriptions,
    SharedMemory,
    Memory,
}

fn wire_error(err: ScriptError) -> WireError {
    match err {
        ScriptError::InvalidArgument(detail) => WireError::InvalidArgument(detail.into_owned()),
        ScriptError::NotFound => WireError::NotFound,
        ScriptError::NotReady => WireError::NotReady,
        ScriptError::Permission(api) => {
            wire_permission(api).map_or(WireError::Forbidden, WireError::Permission)
        }
        ScriptError::Forbidden => WireError::Forbidden,
        ScriptError::RateLimited(flow) => WireError::RateLimited(match flow {
            Flow::BlobUpload => WireRate::Upload,
            Flow::CreateDocument => WireRate::CreateDocument,
            Flow::CreatePrim => WireRate::CreatePrim,
            Flow::Emit => WireRate::Emit,
            Flow::PortalOpen => WireRate::Portal,
        }),
        ScriptError::LimitReached(stock) => WireError::LimitReached(match stock {
            Stock::Documents => WireLimit::Documents,
            Stock::SessionMemory => WireLimit::SharedMemory,
            Stock::Prims => WireLimit::Prims,
            Stock::Receptors => WireLimit::Subscriptions,
            Stock::Slots => WireLimit::Handles,
            Stock::WasmMemory => WireLimit::Memory,
        }),
        ScriptError::Internal(detail) => WireError::Internal(detail.into_owned()),
        // Handled by `raise` before this is reached.
        ScriptError::InvalidHandle => WireError::Internal("invalid handle".into()),
    }
}

/// `None` for a permission outside the protocol, such as node storage: `host
/// ::storage::require` already lowers its own denial to [`ScriptError::
/// Forbidden`], so this is never actually reached for [`HostApi::Storage`].
const fn wire_permission(api: HostApi) -> Option<WirePermission> {
    Some(match api {
        HostApi::Scene => WirePermission::Scene,
        HostApi::CreateDocument => WirePermission::CreateDocument,
        HostApi::Commit => WirePermission::Commit,
        HostApi::Event => WirePermission::Event,
        HostApi::Input => WirePermission::Input,
        HostApi::InputContext => WirePermission::InputDevice,
        HostApi::LocalAgent => WirePermission::LocalAgent,
        HostApi::Identity => WirePermission::Identity,
        HostApi::Peer => WirePermission::Peer,
        HostApi::Physics => WirePermission::Physics,
        HostApi::Portal => WirePermission::Portal,
        HostApi::Travel => WirePermission::Travel,
        HostApi::Storage => return None,
    })
}

/// `wired:script/host.granted`'s argument: a bare kebab-case string.
pub fn permission(value: &str) -> Result<HostApi, ScriptError> {
    Ok(match value {
        "scene" => HostApi::Scene,
        "create-document" => HostApi::CreateDocument,
        "commit" => HostApi::Commit,
        "event" => HostApi::Event,
        "input" => HostApi::Input,
        "input-device" => HostApi::InputContext,
        "local-agent" => HostApi::LocalAgent,
        "identity" => HostApi::Identity,
        "peer" => HostApi::Peer,
        "physics" => HostApi::Physics,
        "portal" => HostApi::Portal,
        "travel" => HostApi::Travel,
        _ => return Err(ScriptError::invalid("not a permission")),
    })
}

/// Serializes through a `Serializer` that lowers `u64`/`i64` as `bigint`,
/// matching how jco lowers a WIT `u64`. The default would lower a document or
/// prim id's words as JS numbers, silently losing precision past 2^53.
pub fn to_js<T: Serialize + ?Sized>(value: &T) -> JsValue {
    let serializer =
        serde_wasm_bindgen::Serializer::new().serialize_large_number_types_as_bigints(true);
    value
        .serialize(&serializer)
        .unwrap_or_else(|err| trap(&err).into())
}

pub fn from_js<T: DeserializeOwned>(value: JsValue) -> Result<T, ScriptError> {
    serde_wasm_bindgen::from_value(value).map_err(|err| ScriptError::invalid(err.to_string()))
}

fn words<const N: usize>(bytes: &[u8]) -> [u64; N] {
    let mut out = [0; N];
    for (word, chunk) in out.iter_mut().zip(bytes.as_chunks::<8>().0) {
        *word = u64::from_le_bytes(*chunk);
    }
    out
}

fn bytes<const N: usize, const W: usize>(words: [u64; W]) -> [u8; N] {
    let mut out = [0; N];
    for (chunk, word) in out.as_chunks_mut::<8>().0.iter_mut().zip(words) {
        *chunk = word.to_le_bytes();
    }
    out
}

pub fn doc_id(value: JsValue) -> Result<DocId, ScriptError> {
    let words: (u64, u64, u64, u64) = from_js(value)?;
    Ok(doc_id_from_words(words))
}

/// For a value already deserialized as part of a larger `serde`-derived
/// shape, such as `ray-filter.exclude-documents` or `link-intent`, whose
/// fields a dedicated wire struct reads as plain `u64` tuples.
pub fn doc_id_from_words(words: (u64, u64, u64, u64)) -> DocId {
    DocId(bytes(words.into()))
}

pub fn wit_doc_id(id: DocId) -> JsValue {
    to_js(&words::<4>(&id.0))
}

pub fn prim_id(value: JsValue) -> Result<PrimId, ScriptError> {
    let words: (u64, u64) = from_js(value)?;
    Ok(PrimId(bytes(words.into())))
}

pub fn wit_prim_id(id: PrimId) -> JsValue {
    to_js(&words::<2>(&id.0))
}

/// See [`doc_id_from_words`].
pub fn link_id_from_words(words: (u64, u64)) -> LinkId {
    LinkId(bytes(words.into()))
}

pub fn wit_link_id(id: LinkId) -> JsValue {
    to_js(&words::<2>(&id.0))
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct WirePrimRef {
    document: (u64, u64, u64, u64),
    prim:     (u64, u64),
}

pub fn prim_ref(value: JsValue) -> Result<(DocId, PrimId), ScriptError> {
    let r: WirePrimRef = from_js(value)?;
    Ok((
        DocId(bytes(r.document.into())),
        PrimId(bytes(r.prim.into())),
    ))
}

pub fn wit_prim_ref((doc, prim): (DocId, PrimId)) -> JsValue {
    to_js(&WirePrimRef {
        document: words(&doc.0).into(),
        prim:     words(&prim.0).into(),
    })
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct WireVec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct WireVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
struct WireQuat {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
struct WireTransform {
    translation: WireVec3,
    rotation:    WireQuat,
    scale:       WireVec3,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct WireColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub fn vec3(value: JsValue) -> Result<bevy::math::Vec3, ScriptError> {
    let v: WireVec3 = from_js(value)?;
    Ok(bevy::math::Vec3::new(v.x, v.y, v.z))
}

pub fn wit_vec3(v: bevy::math::Vec3) -> JsValue {
    to_js(&WireVec3 {
        x: v.x,
        y: v.y,
        z: v.z,
    })
}

pub fn wit_vec2(v: bevy::math::Vec2) -> JsValue {
    to_js(&WireVec2 { x: v.x, y: v.y })
}

pub fn ray(value: JsValue) -> Result<(bevy::math::Vec3, bevy::math::Vec3), ScriptError> {
    #[derive(Deserialize)]
    struct WireRay {
        origin:    WireVec3,
        direction: WireVec3,
    }
    let r: WireRay = from_js(value)?;
    Ok((
        bevy::math::Vec3::new(r.origin.x, r.origin.y, r.origin.z),
        bevy::math::Vec3::new(r.direction.x, r.direction.y, r.direction.z),
    ))
}

pub fn wit_ray(origin: bevy::math::Vec3, direction: bevy::math::Vec3) -> JsValue {
    #[derive(Serialize)]
    struct WireRay {
        origin:    WireVec3,
        direction: WireVec3,
    }
    let v3 = |v: bevy::math::Vec3| WireVec3 {
        x: v.x,
        y: v.y,
        z: v.z,
    };
    to_js(&WireRay {
        origin:    v3(origin),
        direction: v3(direction),
    })
}

pub fn xform(value: JsValue) -> Result<XformAttr, ScriptError> {
    let t: WireTransform = from_js(value)?;
    Ok(XformAttr {
        translation: [t.translation.x, t.translation.y, t.translation.z],
        rotation:    [t.rotation.x, t.rotation.y, t.rotation.z, t.rotation.w],
        scale:       [t.scale.x, t.scale.y, t.scale.z],
    })
}

pub fn wit_xform(t: XformAttr) -> JsValue {
    to_js(&WireTransform {
        translation: WireVec3 {
            x: t.translation[0],
            y: t.translation[1],
            z: t.translation[2],
        },
        rotation:    WireQuat {
            x: t.rotation[0],
            y: t.rotation[1],
            z: t.rotation[2],
            w: t.rotation[3],
        },
        scale:       WireVec3 {
            x: t.scale[0],
            y: t.scale[1],
            z: t.scale[2],
        },
    })
}

pub fn wit_transform(t: bevy::prelude::Transform) -> JsValue {
    to_js(&WireTransform {
        translation: WireVec3 {
            x: t.translation.x,
            y: t.translation.y,
            z: t.translation.z,
        },
        rotation:    WireQuat {
            x: t.rotation.x,
            y: t.rotation.y,
            z: t.rotation.z,
            w: t.rotation.w,
        },
        scale:       WireVec3 {
            x: t.scale.x,
            y: t.scale.y,
            z: t.scale.z,
        },
    })
}

pub fn layer(value: &str) -> Layer {
    match value {
        "shared" => Layer::Shared,
        _ => Layer::Local,
    }
}

/// A tagged value's `tag` string, `""` for one jco lowered without a `tag`
/// key at all (never valid, so every match below falls through to an error).
pub fn tag(value: &JsValue) -> String {
    Reflect::get(value, &JsValue::from_str("tag"))
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

pub fn val(value: &JsValue) -> JsValue {
    Reflect::get(value, &JsValue::from_str("val")).unwrap_or(JsValue::UNDEFINED)
}

fn f32s(value: &JsValue) -> Vec<f32> {
    Float32Array::new(value).to_vec()
}

fn wit_f32s(values: &[f32]) -> JsValue {
    Float32Array::from(values).into()
}

fn u32s(value: &JsValue) -> Vec<u32> {
    Uint32Array::new(value).to_vec()
}

fn wit_u32s(values: &[u32]) -> JsValue {
    Uint32Array::from(values).into()
}

fn bytes_field(value: &JsValue) -> Vec<u8> {
    js_sys::Uint8Array::new(value).to_vec()
}

fn wit_bytes(values: &[u8]) -> JsValue {
    js_sys::Uint8Array::from(values).into()
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum WireTopology {
    PointList,
    LineList,
    LineStrip,
    TriangleList,
    TriangleStrip,
}

const fn topology(t: WireTopology) -> Topology {
    match t {
        WireTopology::PointList => Topology::PointList,
        WireTopology::LineList => Topology::LineList,
        WireTopology::LineStrip => Topology::LineStrip,
        WireTopology::TriangleList => Topology::TriangleList,
        WireTopology::TriangleStrip => Topology::TriangleStrip,
    }
}

const fn wit_topology(t: Topology) -> WireTopology {
    match t {
        Topology::PointList => WireTopology::PointList,
        Topology::LineList => WireTopology::LineList,
        Topology::LineStrip => WireTopology::LineStrip,
        Topology::TriangleList => WireTopology::TriangleList,
        Topology::TriangleStrip => WireTopology::TriangleStrip,
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum WireVertexAttribute {
    Position,
    Normal,
    Tangent,
    Color,
    Uv0,
    Uv1,
}

const fn vertex_attribute(a: WireVertexAttribute) -> VertexAttribute {
    match a {
        WireVertexAttribute::Position => VertexAttribute::Position,
        WireVertexAttribute::Normal => VertexAttribute::Normal,
        WireVertexAttribute::Tangent => VertexAttribute::Tangent,
        WireVertexAttribute::Color => VertexAttribute::Color,
        WireVertexAttribute::Uv0 => VertexAttribute::Uv0,
        WireVertexAttribute::Uv1 => VertexAttribute::Uv1,
    }
}

const fn wit_vertex_attribute(a: VertexAttribute) -> WireVertexAttribute {
    match a {
        VertexAttribute::Position => WireVertexAttribute::Position,
        VertexAttribute::Normal => WireVertexAttribute::Normal,
        VertexAttribute::Tangent => WireVertexAttribute::Tangent,
        VertexAttribute::Color => WireVertexAttribute::Color,
        VertexAttribute::Uv0 => WireVertexAttribute::Uv0,
        VertexAttribute::Uv1 => WireVertexAttribute::Uv1,
    }
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "tag", content = "val", rename_all = "kebab-case")]
enum WireRelation {
    ShaderBinding,
    BaseColorTexture,
    EmissiveTexture,
    MetallicRoughnessTexture,
    NormalTexture,
    OcclusionTexture,
    ShaderTexture(u8),
    Custom(String),
}

fn relation(value: JsValue) -> Result<Relation, ScriptError> {
    let wire: WireRelation = from_js(value)?;
    Ok(match wire {
        WireRelation::ShaderBinding => Relation::ShaderBinding,
        WireRelation::BaseColorTexture => Relation::BaseColorTexture,
        WireRelation::EmissiveTexture => Relation::EmissiveTexture,
        WireRelation::MetallicRoughnessTexture => Relation::MetallicRoughnessTexture,
        WireRelation::NormalTexture => Relation::NormalTexture,
        WireRelation::OcclusionTexture => Relation::OcclusionTexture,
        WireRelation::ShaderTexture(slot) => Relation::ShaderTexture(slot),
        WireRelation::Custom(name) => Relation::Custom(custom_name(&name)?),
    })
}

fn wit_relation(r: Relation) -> JsValue {
    to_js(&match r {
        Relation::ShaderBinding => WireRelation::ShaderBinding,
        Relation::BaseColorTexture => WireRelation::BaseColorTexture,
        Relation::EmissiveTexture => WireRelation::EmissiveTexture,
        Relation::MetallicRoughnessTexture => WireRelation::MetallicRoughnessTexture,
        Relation::NormalTexture => WireRelation::NormalTexture,
        Relation::OcclusionTexture => WireRelation::OcclusionTexture,
        Relation::ShaderTexture(slot) => WireRelation::ShaderTexture(slot),
        Relation::Custom(name) => WireRelation::Custom(name.to_string()),
    })
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct WireMaterial {
    base_color:   Option<WireColor>,
    emissive:     Option<WireColor>,
    metallic:     Option<f32>,
    roughness:    Option<f32>,
    alpha_mode:   Option<WireAlphaMode>,
    alpha_cutoff: Option<f32>,
    double_sided: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum WireAlphaMode {
    Opaque,
    Mask,
    Blend,
    Add,
    Multiply,
    Premultiplied,
}

fn color_vec(c: WireColor) -> ColorVec {
    ColorVec(vec![
        f64::from(c.r),
        f64::from(c.g),
        f64::from(c.b),
        f64::from(c.a),
    ])
}

/// Missing channels read as 1, matching native.
fn wire_color(c: &ColorVec) -> WireColor {
    let channel = |i: usize| c.0.get(i).copied().unwrap_or(1.0) as f32;
    WireColor {
        r: channel(0),
        g: channel(1),
        b: channel(2),
        a: channel(3),
    }
}

fn material(value: JsValue) -> Result<MaterialAttr, ScriptError> {
    let m: WireMaterial = from_js(value)?;
    Ok(MaterialAttr {
        alpha_cutoff: m.alpha_cutoff.map(f64::from),
        alpha_mode:   m.alpha_mode.map(|mode| match mode {
            WireAlphaMode::Opaque => AlphaMode::Opaque,
            WireAlphaMode::Mask => AlphaMode::Mask,
            WireAlphaMode::Blend => AlphaMode::Blend,
            WireAlphaMode::Add => AlphaMode::Add,
            WireAlphaMode::Multiply => AlphaMode::Multiply,
            WireAlphaMode::Premultiplied => AlphaMode::Premultiplied,
        }),
        base_color:   m.base_color.map(color_vec),
        double_sided: m.double_sided,
        emissive:     m.emissive.map(color_vec),
        metallic:     m.metallic.map(f64::from),
        roughness:    m.roughness.map(f64::from),
    })
}

fn wit_material(m: &MaterialAttr) -> JsValue {
    to_js(&WireMaterial {
        base_color:   m.base_color.as_ref().map(wire_color),
        emissive:     m.emissive.as_ref().map(wire_color),
        metallic:     m.metallic.map(|v| v as f32),
        roughness:    m.roughness.map(|v| v as f32),
        alpha_mode:   m.alpha_mode.map(|mode| match mode {
            AlphaMode::Opaque => WireAlphaMode::Opaque,
            AlphaMode::Mask => WireAlphaMode::Mask,
            AlphaMode::Blend => WireAlphaMode::Blend,
            AlphaMode::Add => WireAlphaMode::Add,
            AlphaMode::Multiply => WireAlphaMode::Multiply,
            AlphaMode::Premultiplied => WireAlphaMode::Premultiplied,
        }),
        alpha_cutoff: m.alpha_cutoff.map(|v| v as f32),
        double_sided: m.double_sided,
    })
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum WireAddressMode {
    Repeat,
    MirrorRepeat,
    ClampToEdge,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum WireFilterMode {
    Linear,
    Nearest,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct WireImageSampler {
    address_mode_u: Option<WireAddressMode>,
    address_mode_v: Option<WireAddressMode>,
    address_mode_w: Option<WireAddressMode>,
    mag_filter:     Option<WireFilterMode>,
    min_filter:     Option<WireFilterMode>,
    mipmap_filter:  Option<WireFilterMode>,
    srgb:           Option<bool>,
}

fn image_sampler(value: JsValue) -> Result<ImageSampler, ScriptError> {
    let s: WireImageSampler = from_js(value)?;
    let address = |m: WireAddressMode| match m {
        WireAddressMode::Repeat => AddressMode::Repeat,
        WireAddressMode::MirrorRepeat => AddressMode::MirrorRepeat,
        WireAddressMode::ClampToEdge => AddressMode::ClampToEdge,
    };
    let filter = |m: WireFilterMode| match m {
        WireFilterMode::Linear => FilterMode::Linear,
        WireFilterMode::Nearest => FilterMode::Nearest,
    };
    Ok(ImageSampler {
        address_mode_u: s.address_mode_u.map(address),
        address_mode_v: s.address_mode_v.map(address),
        address_mode_w: s.address_mode_w.map(address),
        mag_filter:     s.mag_filter.map(filter),
        min_filter:     s.min_filter.map(filter),
        mipmap_filter:  s.mipmap_filter.map(filter),
        srgb:           s.srgb,
    })
}

fn wit_image_sampler(s: ImageSampler) -> JsValue {
    let address = |m: AddressMode| match m {
        AddressMode::Repeat => WireAddressMode::Repeat,
        AddressMode::MirrorRepeat => WireAddressMode::MirrorRepeat,
        AddressMode::ClampToEdge => WireAddressMode::ClampToEdge,
    };
    let filter = |m: FilterMode| match m {
        FilterMode::Linear => WireFilterMode::Linear,
        FilterMode::Nearest => WireFilterMode::Nearest,
    };
    to_js(&WireImageSampler {
        address_mode_u: s.address_mode_u.map(address),
        address_mode_v: s.address_mode_v.map(address),
        address_mode_w: s.address_mode_w.map(address),
        mag_filter:     s.mag_filter.map(filter),
        min_filter:     s.min_filter.map(filter),
        mipmap_filter:  s.mipmap_filter.map(filter),
        srgb:           s.srgb,
    })
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(tag = "tag", content = "val", rename_all = "kebab-case")]
enum WireCollider {
    Capsule(WireCapsule),
    ConvexHull,
    Cuboid(WireVec3),
    Cylinder(WireCylinder),
    Sphere(f32),
    Trimesh,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
struct WireCapsule {
    height: f32,
    radius: f32,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
struct WireCylinder {
    height: f32,
    radius: f32,
}

fn collider(value: JsValue) -> Result<ColliderKind, ScriptError> {
    let c: WireCollider = from_js(value)?;
    Ok(match c {
        WireCollider::Capsule(c) => ColliderKind::Capsule {
            height: f64::from(c.height),
            radius: f64::from(c.radius),
        },
        WireCollider::ConvexHull => ColliderKind::ConvexHull,
        WireCollider::Cuboid(size) => ColliderKind::Cuboid {
            x: f64::from(size.x),
            y: f64::from(size.y),
            z: f64::from(size.z),
        },
        WireCollider::Cylinder(c) => ColliderKind::Cylinder {
            height: f64::from(c.height),
            radius: f64::from(c.radius),
        },
        WireCollider::Sphere(radius) => ColliderKind::Sphere(f64::from(radius)),
        WireCollider::Trimesh => ColliderKind::Trimesh,
    })
}

fn wit_collider(c: ColliderKind) -> JsValue {
    to_js(&match c {
        ColliderKind::Capsule { height, radius } => WireCollider::Capsule(WireCapsule {
            height: height as f32,
            radius: radius as f32,
        }),
        ColliderKind::ConvexHull => WireCollider::ConvexHull,
        ColliderKind::Cuboid { x, y, z } => WireCollider::Cuboid(WireVec3 {
            x: x as f32,
            y: y as f32,
            z: z as f32,
        }),
        ColliderKind::Cylinder { height, radius } => WireCollider::Cylinder(WireCylinder {
            height: height as f32,
            radius: radius as f32,
        }),
        ColliderKind::Sphere(radius) => WireCollider::Sphere(radius as f32),
        ColliderKind::Trimesh => WireCollider::Trimesh,
    })
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum WireRigidBodyKind {
    Dynamic,
    Kinematic,
    Static,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct WireRigidBody {
    kind:            WireRigidBodyKind,
    mass:            Option<f32>,
    friction:        Option<f32>,
    restitution:     Option<f32>,
    linear_damping:  Option<f32>,
    angular_damping: Option<f32>,
}

fn rigid_body(value: JsValue) -> Result<RigidBodyAttr, ScriptError> {
    let b: WireRigidBody = from_js(value)?;
    Ok(RigidBodyAttr {
        kind:            Some(match b.kind {
            WireRigidBodyKind::Dynamic => RigidBodyKind::Dynamic,
            WireRigidBodyKind::Kinematic => RigidBodyKind::Kinematic,
            WireRigidBodyKind::Static => RigidBodyKind::Static,
        }),
        mass:            b.mass.map(f64::from),
        friction:        b.friction.map(f64::from),
        restitution:     b.restitution.map(f64::from),
        linear_damping:  b.linear_damping.map(f64::from),
        angular_damping: b.angular_damping.map(f64::from),
    })
}

fn wit_rigid_body(b: RigidBodyAttr) -> JsValue {
    to_js(&WireRigidBody {
        kind:            match b.kind.unwrap_or_default() {
            RigidBodyKind::Dynamic => WireRigidBodyKind::Dynamic,
            RigidBodyKind::Kinematic => WireRigidBodyKind::Kinematic,
            RigidBodyKind::Static => WireRigidBodyKind::Static,
        },
        mass:            b.mass.map(|v| v as f32),
        friction:        b.friction.map(|v| v as f32),
        restitution:     b.restitution.map(|v| v as f32),
        linear_damping:  b.linear_damping.map(|v| v as f32),
        angular_damping: b.angular_damping.map(|v| v as f32),
    })
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct WirePortalDestination {
    space: (u64, u64, u64, u64),
    link:  Option<(u64, u64)>,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
struct WirePortal {
    width:       f32,
    height:      f32,
    destination: Option<WirePortalDestination>,
}

fn portal(value: JsValue) -> Result<PortalAttr, ScriptError> {
    let p: WirePortal = from_js(value)?;
    Ok(PortalAttr {
        destination: p.destination.map(|d| PortalDestination {
            space: bytes(d.space.into()),
            link:  d.link.map(|l| LinkId(bytes(l.into()))),
        }),
        size_x:      f64::from(p.width),
        size_y:      f64::from(p.height),
    })
}

fn wit_portal(p: PortalAttr) -> JsValue {
    to_js(&WirePortal {
        width:       p.size_x as f32,
        height:      p.size_y as f32,
        destination: p.destination.map(|d| WirePortalDestination {
            space: words(&d.space).into(),
            link:  d.link.map(|l| words(&l.0).into()),
        }),
    })
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum WireTextAlign {
    Left,
    Center,
    Right,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum WireTextAnchor {
    Baseline,
    Top,
    Middle,
    Bottom,
}

#[derive(Deserialize, Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
enum WireTextBillboard {
    None,
    Yaw,
    Full,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WireText {
    value:         String,
    size:          Option<f32>,
    align:         Option<WireTextAlign>,
    anchor:        Option<WireTextAnchor>,
    wrap:          Option<f32>,
    line_height:   Option<f32>,
    color:         Option<WireColor>,
    outline:       Option<WireColor>,
    outline_width: Option<f32>,
    emissive:      Option<f32>,
    billboard:     Option<WireTextBillboard>,
}

fn text(value: JsValue) -> Result<TextAttr, ScriptError> {
    let t: WireText = from_js(value)?;
    Ok(TextAttr {
        value:         t.value,
        size:          t.size.map(f64::from),
        align:         t.align.map(|a| match a {
            WireTextAlign::Left => TextAlign::Left,
            WireTextAlign::Center => TextAlign::Center,
            WireTextAlign::Right => TextAlign::Right,
        }),
        anchor:        t.anchor.map(|a| match a {
            WireTextAnchor::Baseline => TextAnchor::Baseline,
            WireTextAnchor::Top => TextAnchor::Top,
            WireTextAnchor::Middle => TextAnchor::Middle,
            WireTextAnchor::Bottom => TextAnchor::Bottom,
        }),
        wrap:          t.wrap.map(f64::from),
        line_height:   t.line_height.map(f64::from),
        color:         t.color.map(color_vec),
        outline:       t.outline.map(color_vec),
        outline_width: t.outline_width.map(f64::from),
        emissive:      t.emissive.map(f64::from),
        billboard:     t.billboard.map(|b| match b {
            WireTextBillboard::None => TextBillboard::None,
            WireTextBillboard::Yaw => TextBillboard::Yaw,
            WireTextBillboard::Full => TextBillboard::Full,
        }),
    })
}

fn wit_text(t: TextAttr) -> JsValue {
    to_js(&WireText {
        value:         t.value,
        size:          t.size.map(|v| v as f32),
        align:         t.align.map(|a| match a {
            TextAlign::Left => WireTextAlign::Left,
            TextAlign::Center => WireTextAlign::Center,
            TextAlign::Right => WireTextAlign::Right,
        }),
        anchor:        t.anchor.map(|a| match a {
            TextAnchor::Baseline => WireTextAnchor::Baseline,
            TextAnchor::Top => WireTextAnchor::Top,
            TextAnchor::Middle => WireTextAnchor::Middle,
            TextAnchor::Bottom => WireTextAnchor::Bottom,
        }),
        wrap:          t.wrap.map(|v| v as f32),
        line_height:   t.line_height.map(|v| v as f32),
        color:         t.color.as_ref().map(wire_color),
        outline:       t.outline.as_ref().map(wire_color),
        outline_width: t.outline_width.map(|v| v as f32),
        emissive:      t.emissive.map(|v| v as f32),
        billboard:     t.billboard.map(|b| match b {
            TextBillboard::None => WireTextBillboard::None,
            TextBillboard::Yaw => WireTextBillboard::Yaw,
            TextBillboard::Full => WireTextBillboard::Full,
        }),
    })
}

/// `wired:scene/properties.property-key`. `tag` alone names most keys; the
/// three carrying a payload (`mesh-vertices`, `relation`, `custom`) read it
/// from `val`.
pub fn property_key(value: &JsValue) -> Result<PropertyKey, ScriptError> {
    Ok(match tag(value).as_str() {
        "parent" => PropertyKey::Parent,
        "name" => PropertyKey::Name,
        "transform" => PropertyKey::Transform,
        "reference" => PropertyKey::Reference,
        "mesh-topology" => PropertyKey::MeshTopology,
        "mesh-vertices" => {
            let a: WireVertexAttribute = from_js(val(value))?;
            PropertyKey::MeshVertices(vertex_attribute(a))
        }
        "mesh-indices" => PropertyKey::MeshIndices,
        "material" => PropertyKey::Material,
        "image-sampler" => PropertyKey::ImageSampler,
        "image-data" => PropertyKey::ImageData,
        "text" => PropertyKey::Text,
        "collider" => PropertyKey::Collider,
        "collider-vertices" => PropertyKey::ColliderVertices,
        "collider-indices" => PropertyKey::ColliderIndices,
        "rigid-body" => PropertyKey::RigidBody,
        "gravity-scale" => PropertyKey::GravityScale,
        "portal" => PropertyKey::Portal,
        "spawn" => PropertyKey::Spawn,
        "relation" => PropertyKey::Relation(relation(val(value))?),
        "custom" => {
            let name = val(value)
                .as_string()
                .ok_or_else(|| ScriptError::invalid("a custom key is a string"))?;
            PropertyKey::Custom(custom_name(&name)?)
        }
        _ => return Err(ScriptError::invalid("not a property key")),
    })
}

fn tagged(tag: &str, val: JsValue) -> JsValue {
    let obj = js_sys::Object::new();
    Reflect::set(&obj, &JsValue::from_str("tag"), &JsValue::from_str(tag)).ok();
    if !val.is_undefined() {
        Reflect::set(&obj, &JsValue::from_str("val"), &val).ok();
    }
    obj.into()
}

pub fn wit_property_key(key: PropertyKey) -> JsValue {
    match key {
        PropertyKey::Parent => tagged("parent", JsValue::UNDEFINED),
        PropertyKey::Name => tagged("name", JsValue::UNDEFINED),
        PropertyKey::Transform => tagged("transform", JsValue::UNDEFINED),
        PropertyKey::Reference => tagged("reference", JsValue::UNDEFINED),
        PropertyKey::MeshTopology => tagged("mesh-topology", JsValue::UNDEFINED),
        PropertyKey::MeshVertices(a) => tagged("mesh-vertices", to_js(&wit_vertex_attribute(a))),
        PropertyKey::MeshIndices => tagged("mesh-indices", JsValue::UNDEFINED),
        PropertyKey::Material => tagged("material", JsValue::UNDEFINED),
        PropertyKey::ImageSampler => tagged("image-sampler", JsValue::UNDEFINED),
        PropertyKey::ImageData => tagged("image-data", JsValue::UNDEFINED),
        PropertyKey::Text => tagged("text", JsValue::UNDEFINED),
        PropertyKey::Collider => tagged("collider", JsValue::UNDEFINED),
        PropertyKey::ColliderVertices => tagged("collider-vertices", JsValue::UNDEFINED),
        PropertyKey::ColliderIndices => tagged("collider-indices", JsValue::UNDEFINED),
        PropertyKey::RigidBody => tagged("rigid-body", JsValue::UNDEFINED),
        PropertyKey::GravityScale => tagged("gravity-scale", JsValue::UNDEFINED),
        PropertyKey::Portal => tagged("portal", JsValue::UNDEFINED),
        PropertyKey::Spawn => tagged("spawn", JsValue::UNDEFINED),
        PropertyKey::Relation(r) => tagged("relation", wit_relation(r)),
        PropertyKey::Custom(name) => tagged("custom", JsValue::from_str(&name.to_string())),
    }
}

/// `wired:scene/properties.property`. The four variants jco lowers as a typed
/// array (`mesh-vertices`, `mesh-indices`, `collider-vertices`,
/// `collider-indices`) and `custom`'s byte half are read and built directly;
/// serde has no vocabulary for a non-byte typed array.
pub fn property(value: &JsValue) -> Result<Property, ScriptError> {
    let val = val(value);
    Ok(match tag(value).as_str() {
        "parent" => Property::Parent(if val.is_undefined() {
            None
        } else {
            Some(prim_id(val)?)
        }),
        "name" => Property::Name(
            val.as_string()
                .ok_or_else(|| ScriptError::invalid("a name is a string"))?,
        ),
        "transform" => Property::Transform(xform(val)?),
        "reference" => Property::Reference(doc_id(val)?),
        "mesh-topology" => Property::MeshTopology(topology(from_js(val)?)),
        "mesh-vertices" => {
            let attribute: WireVertexAttribute = from_js(
                Reflect::get(&val, &JsValue::from_str("attribute"))
                    .map_err(|_| ScriptError::invalid("a vertex stream names its attribute"))?,
            )?;
            let values = Reflect::get(&val, &JsValue::from_str("values"))
                .map_err(|_| ScriptError::invalid("a vertex stream carries its values"))?;
            Property::MeshVertices(vertex_attribute(attribute), f32s(&values))
        }
        "mesh-indices" => Property::MeshIndices(u32s(&val)),
        "material" => Property::Material(material(val)?),
        "image-sampler" => Property::ImageSampler(image_sampler(val)?),
        "image-data" => Property::ImageData(bytes_field(&val)),
        "text" => Property::Text(text(val)?),
        "collider" => Property::Collider(collider(val)?),
        "collider-vertices" => Property::ColliderVertices(f32s(&val)),
        "collider-indices" => Property::ColliderIndices(u32s(&val)),
        "rigid-body" => Property::RigidBody(rigid_body(val)?),
        "gravity-scale" => Property::GravityScale(
            val.as_f64()
                .ok_or_else(|| ScriptError::invalid("a gravity scale is a number"))?
                as f32,
        ),
        "portal" => Property::Portal(portal(val)?),
        "spawn" => {
            #[derive(Deserialize)]
            struct WireSpawn {
                radius: f32,
            }
            let s: WireSpawn = from_js(val)?;
            Property::Spawn(SpawnAttr {
                radius: f64::from(s.radius),
            })
        }
        "relation" => {
            let pair = Array::from(&val);
            let target = prim_id(pair.get(1))?;
            Property::Relation(relation(pair.get(0))?, target)
        }
        "custom" => {
            let pair = Array::from(&val);
            let name = pair
                .get(0)
                .as_string()
                .ok_or_else(|| ScriptError::invalid("a custom name is a string"))?;
            Property::Custom(custom_name(&name)?, bytes_field(&pair.get(1)))
        }
        _ => return Err(ScriptError::invalid("not a property")),
    })
}

pub fn wit_property(value: Property) -> JsValue {
    match value {
        Property::Parent(parent) => {
            tagged("parent", parent.map_or(JsValue::UNDEFINED, wit_prim_id))
        }
        Property::Name(name) => tagged("name", JsValue::from_str(&name)),
        Property::Transform(t) => tagged("transform", wit_xform(t)),
        Property::Reference(doc) => tagged("reference", wit_doc_id(doc)),
        Property::MeshTopology(t) => tagged("mesh-topology", to_js(&wit_topology(t))),
        Property::MeshVertices(a, values) => {
            let obj = js_sys::Object::new();
            Reflect::set(
                &obj,
                &JsValue::from_str("attribute"),
                &to_js(&wit_vertex_attribute(a)),
            )
            .ok();
            Reflect::set(&obj, &JsValue::from_str("values"), &wit_f32s(&values)).ok();
            tagged("mesh-vertices", obj.into())
        }
        Property::MeshIndices(values) => tagged("mesh-indices", wit_u32s(&values)),
        Property::Material(m) => tagged("material", wit_material(&m)),
        Property::ImageSampler(s) => tagged("image-sampler", wit_image_sampler(s)),
        Property::ImageData(bytes) => tagged("image-data", wit_bytes(&bytes)),
        Property::Text(t) => tagged("text", wit_text(t)),
        Property::Collider(c) => tagged("collider", wit_collider(c)),
        Property::ColliderVertices(values) => tagged("collider-vertices", wit_f32s(&values)),
        Property::ColliderIndices(values) => tagged("collider-indices", wit_u32s(&values)),
        Property::RigidBody(b) => tagged("rigid-body", wit_rigid_body(b)),
        Property::GravityScale(scale) => {
            tagged("gravity-scale", JsValue::from_f64(f64::from(scale)))
        }
        Property::Portal(p) => tagged("portal", wit_portal(p)),
        Property::Spawn(s) => {
            #[derive(Serialize)]
            struct WireSpawn {
                radius: f32,
            }
            tagged(
                "spawn",
                to_js(&WireSpawn {
                    radius: s.radius as f32,
                }),
            )
        }
        Property::Relation(r, target) => tagged(
            "relation",
            Array::of2(&wit_relation(r), &wit_prim_id(target)).into(),
        ),
        Property::Custom(name, bytes) => tagged(
            "custom",
            Array::of2(&JsValue::from_str(&name.to_string()), &wit_bytes(&bytes)).into(),
        ),
    }
}

/// `wired:scene/document.edit`.
pub fn edit(value: JsValue) -> Result<Edit, ScriptError> {
    let val = val(&value);
    Ok(match tag(&value).as_str() {
        "set" => {
            let pair = Array::from(&val);
            Edit::Set(prim_id(pair.get(0))?, property(&pair.get(1))?)
        }
        "clear" => {
            let pair = Array::from(&val);
            Edit::Clear(prim_id(pair.get(0))?, property_key(&pair.get(1))?)
        }
        "remove" => Edit::Remove(prim_id(val)?),
        _ => return Err(ScriptError::invalid("not an edit")),
    })
}

/// `wired:scene/document.anchor`. `prim(prim-ref)` names a document by its
/// `document-id`; the caller resolves it against a loaded document, as
/// `Anchor::Prim` needs a live [`DocId`]/[`PrimId`] pair, not a handle.
pub fn anchor(value: JsValue) -> Result<Anchor, ScriptError> {
    match tag(&value).as_str() {
        "space" => Ok(Anchor::Space),
        "prim" => {
            let (doc, prim) = prim_ref(val(&value))?;
            Ok(Anchor::Prim(doc, prim))
        }
        _ => Err(ScriptError::invalid("not an anchor")),
    }
}
