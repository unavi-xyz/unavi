//! Conversions between generated WIT types and host types.

use bevy::prelude::{
    Transform as BevyTransform,
    Vec2 as BevyVec2,
    Vec3 as BevyVec3,
};
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
use unavi_policy::{
    permissions::HostApi,
    quota::{
        Flow,
        Stock,
    },
};

use super::generated::wired::{
    core::{
        error::{
            Error,
            Limit,
            Permission,
            Rate,
        },
        ids::{
            DocumentId,
            PrimRef,
        },
        math::{
            Color,
            Quat,
            Ray,
            Transform,
            Vec2,
            Vec3,
        },
    },
    scene::{
        document::{
            Edit as WitEdit,
            Layer as WitLayer,
        },
        properties as wit,
    },
};
use crate::{
    error::ScriptError,
    host::scene::{
        edit::{
            Edit,
            Layer,
        },
        property::{
            Property,
            PropertyKey,
            Relation,
            VertexAttribute,
            custom_name,
        },
    },
};

/// Lowers a host error. A handle the guest does not hold traps rather than
/// answering, since only a broken guest can pass one.
pub fn error(err: ScriptError) -> wasmtime::Result<Error> {
    Ok(match err {
        ScriptError::InvalidArgument(detail) => Error::InvalidArgument(detail.into_owned()),
        ScriptError::NotFound => Error::NotFound,
        ScriptError::NotReady => Error::NotReady,
        ScriptError::Permission(api) => {
            wit_permission(api).map_or(Error::Forbidden, Error::Permission)
        }
        ScriptError::Forbidden => Error::Forbidden,
        ScriptError::RateLimited(flow) => Error::RateLimited(match flow {
            Flow::BlobUpload => Rate::Upload,
            Flow::CreateDocument => Rate::CreateDocument,
            Flow::CreatePrim => Rate::CreatePrim,
            Flow::Emit => Rate::Emit,
            Flow::PortalOpen => Rate::Portal,
        }),
        ScriptError::LimitReached(stock) => Error::LimitReached(match stock {
            Stock::Documents => Limit::Documents,
            Stock::SessionMemory => Limit::SharedMemory,
            Stock::Prims => Limit::Prims,
            Stock::Receptors => Limit::Subscriptions,
            Stock::Slots => Limit::Handles,
            Stock::WasmMemory => Limit::Memory,
        }),
        ScriptError::Internal(detail) => Error::Internal(detail.into_owned()),
        ScriptError::InvalidHandle => return Err(wasmtime::Error::new(ScriptError::InvalidHandle)),
    })
}

pub const fn permission(permission: Permission) -> HostApi {
    match permission {
        Permission::Scene => HostApi::Scene,
        Permission::CreateDocument => HostApi::CreateDocument,
        Permission::Commit => HostApi::Commit,
        Permission::Event => HostApi::Event,
        Permission::Input => HostApi::Input,
        Permission::InputDevice => HostApi::InputContext,
        Permission::LocalAgent => HostApi::LocalAgent,
        Permission::Identity => HostApi::Identity,
        Permission::Peer => HostApi::Peer,
        Permission::Physics => HostApi::Physics,
        Permission::Portal => HostApi::Portal,
        Permission::Travel => HostApi::Travel,
    }
}

/// `None` for a permission outside the protocol, such as node storage.
const fn wit_permission(api: HostApi) -> Option<Permission> {
    Some(match api {
        HostApi::Scene => Permission::Scene,
        HostApi::CreateDocument => Permission::CreateDocument,
        HostApi::Commit => Permission::Commit,
        HostApi::Event => Permission::Event,
        HostApi::Input => Permission::Input,
        HostApi::InputContext => Permission::InputDevice,
        HostApi::LocalAgent => Permission::LocalAgent,
        HostApi::Identity => Permission::Identity,
        HostApi::Peer => Permission::Peer,
        HostApi::Physics => Permission::Physics,
        HostApi::Portal => Permission::Portal,
        HostApi::Travel => Permission::Travel,
        HostApi::Storage => return None,
    })
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

pub fn doc_id(id: DocumentId) -> DocId {
    DocId(bytes(id.into()))
}

pub fn wit_doc_id(id: DocId) -> DocumentId {
    words::<4>(&id.0).into()
}

pub fn prim_id(id: (u64, u64)) -> PrimId {
    PrimId(bytes(id.into()))
}

pub fn wit_prim_id(id: PrimId) -> (u64, u64) {
    words::<2>(&id.0).into()
}

pub fn link_id(id: (u64, u64)) -> LinkId {
    LinkId(bytes(id.into()))
}

pub fn wit_link_id(id: LinkId) -> (u64, u64) {
    words::<2>(&id.0).into()
}

pub fn prim_ref(r: PrimRef) -> (DocId, PrimId) {
    (doc_id(r.document), prim_id(r.prim))
}

pub fn wit_prim_ref((doc, prim): (DocId, PrimId)) -> PrimRef {
    PrimRef {
        document: wit_doc_id(doc),
        prim:     wit_prim_id(prim),
    }
}

pub const fn wit_vec2(v: BevyVec2) -> Vec2 {
    Vec2 { x: v.x, y: v.y }
}

pub const fn vec3(v: Vec3) -> BevyVec3 {
    BevyVec3::new(v.x, v.y, v.z)
}

pub const fn wit_vec3(v: BevyVec3) -> Vec3 {
    Vec3 {
        x: v.x,
        y: v.y,
        z: v.z,
    }
}

const fn arr3(v: Vec3) -> [f32; 3] {
    [v.x, v.y, v.z]
}

const fn wit_arr3([x, y, z]: [f32; 3]) -> Vec3 {
    Vec3 { x, y, z }
}

pub const fn ray(r: Ray) -> (BevyVec3, BevyVec3) {
    (vec3(r.origin), vec3(r.direction))
}

pub const fn wit_ray(origin: BevyVec3, direction: BevyVec3) -> Ray {
    Ray {
        origin:    wit_vec3(origin),
        direction: wit_vec3(direction),
    }
}

pub const fn xform(t: Transform) -> XformAttr {
    XformAttr {
        translation: arr3(t.translation),
        rotation:    [t.rotation.x, t.rotation.y, t.rotation.z, t.rotation.w],
        scale:       arr3(t.scale),
    }
}

pub const fn wit_xform(t: XformAttr) -> Transform {
    Transform {
        translation: wit_arr3(t.translation),
        rotation:    Quat {
            x: t.rotation[0],
            y: t.rotation[1],
            z: t.rotation[2],
            w: t.rotation[3],
        },
        scale:       wit_arr3(t.scale),
    }
}

pub fn wit_transform(t: BevyTransform) -> Transform {
    Transform {
        translation: wit_vec3(t.translation),
        rotation:    Quat {
            x: t.rotation.x,
            y: t.rotation.y,
            z: t.rotation.z,
            w: t.rotation.w,
        },
        scale:       wit_vec3(t.scale),
    }
}

pub const fn color(c: Color) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}

pub const fn wit_color([r, g, b, a]: [f32; 4]) -> Color {
    Color { r, g, b, a }
}

fn color_vec(c: Color) -> ColorVec {
    ColorVec(color(c).map(f64::from).to_vec())
}

/// Missing channels read as 1.
fn wit_color_vec(c: &ColorVec) -> Color {
    let channel = |i: usize| c.0.get(i).copied().unwrap_or(1.0) as f32;
    wit_color([channel(0), channel(1), channel(2), channel(3)])
}

/// Documents store `f64`; the protocol carries `f32`.
const fn narrow(v: f64) -> f32 {
    v as f32
}

pub const fn layer(layer: WitLayer) -> Layer {
    match layer {
        WitLayer::Local => Layer::Local,
        WitLayer::Shared => Layer::Shared,
    }
}

pub fn edit(edit: WitEdit) -> Result<Edit, ScriptError> {
    Ok(match edit {
        WitEdit::Set((prim, value)) => Edit::Set(prim_id(prim), property(value)?),
        WitEdit::Clear((prim, key)) => Edit::Clear(prim_id(prim), property_key(key)?),
        WitEdit::Remove(prim) => Edit::Remove(prim_id(prim)),
    })
}

const fn vertex_attribute(a: wit::VertexAttribute) -> VertexAttribute {
    match a {
        wit::VertexAttribute::Position => VertexAttribute::Position,
        wit::VertexAttribute::Normal => VertexAttribute::Normal,
        wit::VertexAttribute::Tangent => VertexAttribute::Tangent,
        wit::VertexAttribute::Color => VertexAttribute::Color,
        wit::VertexAttribute::Uv0 => VertexAttribute::Uv0,
        wit::VertexAttribute::Uv1 => VertexAttribute::Uv1,
    }
}

const fn wit_vertex_attribute(a: VertexAttribute) -> wit::VertexAttribute {
    match a {
        VertexAttribute::Position => wit::VertexAttribute::Position,
        VertexAttribute::Normal => wit::VertexAttribute::Normal,
        VertexAttribute::Tangent => wit::VertexAttribute::Tangent,
        VertexAttribute::Color => wit::VertexAttribute::Color,
        VertexAttribute::Uv0 => wit::VertexAttribute::Uv0,
        VertexAttribute::Uv1 => wit::VertexAttribute::Uv1,
    }
}

fn relation(r: wit::Relation) -> Result<Relation, ScriptError> {
    Ok(match r {
        wit::Relation::ShaderBinding => Relation::ShaderBinding,
        wit::Relation::BaseColorTexture => Relation::BaseColorTexture,
        wit::Relation::EmissiveTexture => Relation::EmissiveTexture,
        wit::Relation::MetallicRoughnessTexture => Relation::MetallicRoughnessTexture,
        wit::Relation::NormalTexture => Relation::NormalTexture,
        wit::Relation::OcclusionTexture => Relation::OcclusionTexture,
        wit::Relation::ShaderTexture(slot) => Relation::ShaderTexture(slot),
        wit::Relation::Custom(name) => Relation::Custom(custom_name(&name)?),
    })
}

fn wit_relation(r: Relation) -> wit::Relation {
    match r {
        Relation::ShaderBinding => wit::Relation::ShaderBinding,
        Relation::BaseColorTexture => wit::Relation::BaseColorTexture,
        Relation::EmissiveTexture => wit::Relation::EmissiveTexture,
        Relation::MetallicRoughnessTexture => wit::Relation::MetallicRoughnessTexture,
        Relation::NormalTexture => wit::Relation::NormalTexture,
        Relation::OcclusionTexture => wit::Relation::OcclusionTexture,
        Relation::ShaderTexture(slot) => wit::Relation::ShaderTexture(slot),
        Relation::Custom(name) => wit::Relation::Custom(name.to_string()),
    }
}

pub fn property_key(key: wit::PropertyKey) -> Result<PropertyKey, ScriptError> {
    use wit::PropertyKey as K;
    Ok(match key {
        K::Parent => PropertyKey::Parent,
        K::Name => PropertyKey::Name,
        K::Transform => PropertyKey::Transform,
        K::Reference => PropertyKey::Reference,
        K::MeshTopology => PropertyKey::MeshTopology,
        K::MeshVertices(a) => PropertyKey::MeshVertices(vertex_attribute(a)),
        K::MeshIndices => PropertyKey::MeshIndices,
        K::Material => PropertyKey::Material,
        K::ImageSampler => PropertyKey::ImageSampler,
        K::ImageData => PropertyKey::ImageData,
        K::Text => PropertyKey::Text,
        K::Collider => PropertyKey::Collider,
        K::ColliderVertices => PropertyKey::ColliderVertices,
        K::ColliderIndices => PropertyKey::ColliderIndices,
        K::RigidBody => PropertyKey::RigidBody,
        K::GravityScale => PropertyKey::GravityScale,
        K::Portal => PropertyKey::Portal,
        K::Spawn => PropertyKey::Spawn,
        K::Relation(r) => PropertyKey::Relation(relation(r)?),
        K::Custom(name) => PropertyKey::Custom(custom_name(&name)?),
    })
}

pub fn wit_property_key(key: PropertyKey) -> wit::PropertyKey {
    use wit::PropertyKey as K;
    match key {
        PropertyKey::Parent => K::Parent,
        PropertyKey::Name => K::Name,
        PropertyKey::Transform => K::Transform,
        PropertyKey::Reference => K::Reference,
        PropertyKey::MeshTopology => K::MeshTopology,
        PropertyKey::MeshVertices(a) => K::MeshVertices(wit_vertex_attribute(a)),
        PropertyKey::MeshIndices => K::MeshIndices,
        PropertyKey::Material => K::Material,
        PropertyKey::ImageSampler => K::ImageSampler,
        PropertyKey::ImageData => K::ImageData,
        PropertyKey::Text => K::Text,
        PropertyKey::Collider => K::Collider,
        PropertyKey::ColliderVertices => K::ColliderVertices,
        PropertyKey::ColliderIndices => K::ColliderIndices,
        PropertyKey::RigidBody => K::RigidBody,
        PropertyKey::GravityScale => K::GravityScale,
        PropertyKey::Portal => K::Portal,
        PropertyKey::Spawn => K::Spawn,
        PropertyKey::Relation(r) => K::Relation(wit_relation(r)),
        PropertyKey::Custom(name) => K::Custom(name.to_string()),
    }
}

pub fn property(value: wit::Property) -> Result<Property, ScriptError> {
    use wit::Property as P;
    Ok(match value {
        P::Parent(parent) => Property::Parent(parent.map(prim_id)),
        P::Name(name) => Property::Name(name),
        P::Transform(t) => Property::Transform(xform(t)),
        P::Reference(doc) => Property::Reference(doc_id(doc)),
        P::MeshTopology(t) => Property::MeshTopology(topology(t)),
        P::MeshVertices(stream) => {
            Property::MeshVertices(vertex_attribute(stream.attribute), stream.values)
        }
        P::MeshIndices(values) => Property::MeshIndices(values),
        P::Material(m) => Property::Material(material(m)),
        P::ImageSampler(s) => Property::ImageSampler(image_sampler(s)),
        P::ImageData(bytes) => Property::ImageData(bytes),
        P::Text(t) => Property::Text(text(t)),
        P::Collider(c) => Property::Collider(collider(c)),
        P::ColliderVertices(values) => Property::ColliderVertices(values),
        P::ColliderIndices(values) => Property::ColliderIndices(values),
        P::RigidBody(b) => Property::RigidBody(rigid_body(b)),
        P::GravityScale(scale) => Property::GravityScale(scale),
        P::Portal(p) => Property::Portal(portal(p)),
        P::Spawn(s) => Property::Spawn(SpawnAttr {
            radius: f64::from(s.radius),
        }),
        P::Relation((r, target)) => Property::Relation(relation(r)?, prim_id(target)),
        P::Custom((name, bytes)) => Property::Custom(custom_name(&name)?, bytes),
    })
}

pub fn wit_property(value: Property) -> wit::Property {
    use wit::Property as P;
    match value {
        Property::Parent(parent) => P::Parent(parent.map(wit_prim_id)),
        Property::Name(name) => P::Name(name),
        Property::Transform(t) => P::Transform(wit_xform(t)),
        Property::Reference(doc) => P::Reference(wit_doc_id(doc)),
        Property::MeshTopology(t) => P::MeshTopology(wit_topology(t)),
        Property::MeshVertices(a, values) => P::MeshVertices(wit::VertexStream {
            attribute: wit_vertex_attribute(a),
            values,
        }),
        Property::MeshIndices(values) => P::MeshIndices(values),
        Property::Material(m) => P::Material(wit_material(&m)),
        Property::ImageSampler(s) => P::ImageSampler(wit_image_sampler(s)),
        Property::ImageData(bytes) => P::ImageData(bytes),
        Property::Text(t) => P::Text(wit_text(t)),
        Property::Collider(c) => P::Collider(wit_collider(c)),
        Property::ColliderVertices(values) => P::ColliderVertices(values),
        Property::ColliderIndices(values) => P::ColliderIndices(values),
        Property::RigidBody(b) => P::RigidBody(wit_rigid_body(b)),
        Property::GravityScale(scale) => P::GravityScale(scale),
        Property::Portal(p) => P::Portal(wit_portal(p)),
        Property::Spawn(s) => P::Spawn(wit::Spawn {
            radius: s.radius as f32,
        }),
        Property::Relation(r, target) => P::Relation((wit_relation(r), wit_prim_id(target))),
        Property::Custom(name, bytes) => P::Custom((name.to_string(), bytes)),
    }
}

const fn topology(t: wit::Topology) -> Topology {
    match t {
        wit::Topology::PointList => Topology::PointList,
        wit::Topology::LineList => Topology::LineList,
        wit::Topology::LineStrip => Topology::LineStrip,
        wit::Topology::TriangleList => Topology::TriangleList,
        wit::Topology::TriangleStrip => Topology::TriangleStrip,
    }
}

const fn wit_topology(t: Topology) -> wit::Topology {
    match t {
        Topology::PointList => wit::Topology::PointList,
        Topology::LineList => wit::Topology::LineList,
        Topology::LineStrip => wit::Topology::LineStrip,
        Topology::TriangleList => wit::Topology::TriangleList,
        Topology::TriangleStrip => wit::Topology::TriangleStrip,
    }
}

fn material(m: wit::Material) -> MaterialAttr {
    MaterialAttr {
        alpha_cutoff: m.alpha_cutoff.map(f64::from),
        alpha_mode:   m.alpha_mode.map(|mode| match mode {
            wit::AlphaMode::Opaque => AlphaMode::Opaque,
            wit::AlphaMode::Mask => AlphaMode::Mask,
            wit::AlphaMode::Blend => AlphaMode::Blend,
            wit::AlphaMode::Add => AlphaMode::Add,
            wit::AlphaMode::Multiply => AlphaMode::Multiply,
            wit::AlphaMode::Premultiplied => AlphaMode::Premultiplied,
        }),
        base_color:   m.base_color.map(color_vec),
        double_sided: m.double_sided,
        emissive:     m.emissive.map(color_vec),
        metallic:     m.metallic.map(f64::from),
        roughness:    m.roughness.map(f64::from),
    }
}

fn wit_material(m: &MaterialAttr) -> wit::Material {
    wit::Material {
        base_color:   m.base_color.as_ref().map(wit_color_vec),
        emissive:     m.emissive.as_ref().map(wit_color_vec),
        metallic:     m.metallic.map(narrow),
        roughness:    m.roughness.map(narrow),
        alpha_mode:   m.alpha_mode.map(|mode| match mode {
            AlphaMode::Opaque => wit::AlphaMode::Opaque,
            AlphaMode::Mask => wit::AlphaMode::Mask,
            AlphaMode::Blend => wit::AlphaMode::Blend,
            AlphaMode::Add => wit::AlphaMode::Add,
            AlphaMode::Multiply => wit::AlphaMode::Multiply,
            AlphaMode::Premultiplied => wit::AlphaMode::Premultiplied,
        }),
        alpha_cutoff: m.alpha_cutoff.map(narrow),
        double_sided: m.double_sided,
    }
}

const fn address_mode(m: wit::AddressMode) -> AddressMode {
    match m {
        wit::AddressMode::Repeat => AddressMode::Repeat,
        wit::AddressMode::MirrorRepeat => AddressMode::MirrorRepeat,
        wit::AddressMode::ClampToEdge => AddressMode::ClampToEdge,
    }
}

const fn wit_address_mode(m: AddressMode) -> wit::AddressMode {
    match m {
        AddressMode::Repeat => wit::AddressMode::Repeat,
        AddressMode::MirrorRepeat => wit::AddressMode::MirrorRepeat,
        AddressMode::ClampToEdge => wit::AddressMode::ClampToEdge,
    }
}

const fn filter_mode(m: wit::FilterMode) -> FilterMode {
    match m {
        wit::FilterMode::Linear => FilterMode::Linear,
        wit::FilterMode::Nearest => FilterMode::Nearest,
    }
}

const fn wit_filter_mode(m: FilterMode) -> wit::FilterMode {
    match m {
        FilterMode::Linear => wit::FilterMode::Linear,
        FilterMode::Nearest => wit::FilterMode::Nearest,
    }
}

fn image_sampler(s: wit::ImageSampler) -> ImageSampler {
    ImageSampler {
        address_mode_u: s.address_mode_u.map(address_mode),
        address_mode_v: s.address_mode_v.map(address_mode),
        address_mode_w: s.address_mode_w.map(address_mode),
        mag_filter:     s.mag_filter.map(filter_mode),
        min_filter:     s.min_filter.map(filter_mode),
        mipmap_filter:  s.mipmap_filter.map(filter_mode),
        srgb:           s.srgb,
    }
}

fn wit_image_sampler(s: ImageSampler) -> wit::ImageSampler {
    wit::ImageSampler {
        address_mode_u: s.address_mode_u.map(wit_address_mode),
        address_mode_v: s.address_mode_v.map(wit_address_mode),
        address_mode_w: s.address_mode_w.map(wit_address_mode),
        mag_filter:     s.mag_filter.map(wit_filter_mode),
        min_filter:     s.min_filter.map(wit_filter_mode),
        mipmap_filter:  s.mipmap_filter.map(wit_filter_mode),
        srgb:           s.srgb,
    }
}

fn text(t: wit::Text) -> TextAttr {
    TextAttr {
        value:         t.value,
        size:          t.size.map(f64::from),
        align:         t.align.map(|a| match a {
            wit::TextAlign::Left => TextAlign::Left,
            wit::TextAlign::Center => TextAlign::Center,
            wit::TextAlign::Right => TextAlign::Right,
        }),
        anchor:        t.anchor.map(|a| match a {
            wit::TextAnchor::Baseline => TextAnchor::Baseline,
            wit::TextAnchor::Top => TextAnchor::Top,
            wit::TextAnchor::Middle => TextAnchor::Middle,
            wit::TextAnchor::Bottom => TextAnchor::Bottom,
        }),
        wrap:          t.wrap.map(f64::from),
        line_height:   t.line_height.map(f64::from),
        color:         t.color.map(color_vec),
        outline:       t.outline.map(color_vec),
        outline_width: t.outline_width.map(f64::from),
        emissive:      t.emissive.map(f64::from),
        billboard:     t.billboard.map(|b| match b {
            wit::TextBillboard::None => TextBillboard::None,
            wit::TextBillboard::Yaw => TextBillboard::Yaw,
            wit::TextBillboard::Full => TextBillboard::Full,
        }),
    }
}

fn wit_text(t: TextAttr) -> wit::Text {
    wit::Text {
        value:         t.value,
        size:          t.size.map(narrow),
        align:         t.align.map(|a| match a {
            TextAlign::Left => wit::TextAlign::Left,
            TextAlign::Center => wit::TextAlign::Center,
            TextAlign::Right => wit::TextAlign::Right,
        }),
        anchor:        t.anchor.map(|a| match a {
            TextAnchor::Baseline => wit::TextAnchor::Baseline,
            TextAnchor::Top => wit::TextAnchor::Top,
            TextAnchor::Middle => wit::TextAnchor::Middle,
            TextAnchor::Bottom => wit::TextAnchor::Bottom,
        }),
        wrap:          t.wrap.map(narrow),
        line_height:   t.line_height.map(narrow),
        color:         t.color.as_ref().map(wit_color_vec),
        outline:       t.outline.as_ref().map(wit_color_vec),
        outline_width: t.outline_width.map(narrow),
        emissive:      t.emissive.map(narrow),
        billboard:     t.billboard.map(|b| match b {
            TextBillboard::None => wit::TextBillboard::None,
            TextBillboard::Yaw => wit::TextBillboard::Yaw,
            TextBillboard::Full => wit::TextBillboard::Full,
        }),
    }
}

fn collider(c: wit::Collider) -> ColliderKind {
    match c {
        wit::Collider::Capsule(c) => ColliderKind::Capsule {
            height: f64::from(c.height),
            radius: f64::from(c.radius),
        },
        wit::Collider::ConvexHull => ColliderKind::ConvexHull,
        wit::Collider::Cuboid(size) => ColliderKind::Cuboid {
            x: f64::from(size.x),
            y: f64::from(size.y),
            z: f64::from(size.z),
        },
        wit::Collider::Cylinder(c) => ColliderKind::Cylinder {
            height: f64::from(c.height),
            radius: f64::from(c.radius),
        },
        wit::Collider::Sphere(radius) => ColliderKind::Sphere(f64::from(radius)),
        wit::Collider::Trimesh => ColliderKind::Trimesh,
    }
}

const fn wit_collider(c: ColliderKind) -> wit::Collider {
    match c {
        ColliderKind::Capsule { height, radius } => wit::Collider::Capsule(wit::ColliderCapsule {
            height: height as f32,
            radius: radius as f32,
        }),
        ColliderKind::ConvexHull => wit::Collider::ConvexHull,
        ColliderKind::Cuboid { x, y, z } => wit::Collider::Cuboid(Vec3 {
            x: x as f32,
            y: y as f32,
            z: z as f32,
        }),
        ColliderKind::Cylinder { height, radius } => {
            wit::Collider::Cylinder(wit::ColliderCylinder {
                height: height as f32,
                radius: radius as f32,
            })
        }
        ColliderKind::Sphere(radius) => wit::Collider::Sphere(radius as f32),
        ColliderKind::Trimesh => wit::Collider::Trimesh,
    }
}

fn rigid_body(b: wit::RigidBody) -> RigidBodyAttr {
    RigidBodyAttr {
        kind:            Some(match b.kind {
            wit::RigidBodyKind::Dynamic => RigidBodyKind::Dynamic,
            wit::RigidBodyKind::Kinematic => RigidBodyKind::Kinematic,
            wit::RigidBodyKind::Static => RigidBodyKind::Static,
        }),
        mass:            b.mass.map(f64::from),
        friction:        b.friction.map(f64::from),
        restitution:     b.restitution.map(f64::from),
        linear_damping:  b.linear_damping.map(f64::from),
        angular_damping: b.angular_damping.map(f64::from),
    }
}

fn wit_rigid_body(b: RigidBodyAttr) -> wit::RigidBody {
    wit::RigidBody {
        kind:            match b.kind.unwrap_or_default() {
            RigidBodyKind::Dynamic => wit::RigidBodyKind::Dynamic,
            RigidBodyKind::Kinematic => wit::RigidBodyKind::Kinematic,
            RigidBodyKind::Static => wit::RigidBodyKind::Static,
        },
        mass:            b.mass.map(narrow),
        friction:        b.friction.map(narrow),
        restitution:     b.restitution.map(narrow),
        linear_damping:  b.linear_damping.map(narrow),
        angular_damping: b.angular_damping.map(narrow),
    }
}

fn portal(p: wit::Portal) -> PortalAttr {
    PortalAttr {
        destination: p.destination.map(|d| PortalDestination {
            space: doc_id(d.space).0,
            link:  d.link.map(link_id),
        }),
        size_x:      f64::from(p.width),
        size_y:      f64::from(p.height),
    }
}

fn wit_portal(p: PortalAttr) -> wit::Portal {
    wit::Portal {
        width:       p.size_x as f32,
        height:      p.size_y as f32,
        destination: p.destination.map(|d| wit::PortalDestination {
            space: wit_doc_id(DocId(d.space)),
            link:  d.link.map(wit_link_id),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_round_trip_through_their_words() {
        let doc = DocId(std::array::from_fn(|i| i as u8));
        assert_eq!(doc_id(wit_doc_id(doc)), doc);
        let prim = PrimId::new();
        assert_eq!(prim_id(wit_prim_id(prim)), prim);
    }

    #[test]
    fn the_first_word_holds_the_first_bytes() {
        let doc = DocId(std::array::from_fn(|i| i as u8));
        assert_eq!(
            wit_doc_id(doc).0,
            u64::from_le_bytes([0, 1, 2, 3, 4, 5, 6, 7])
        );
    }

    #[test]
    fn an_invalid_handle_traps() {
        assert!(error(ScriptError::InvalidHandle).is_err());
        assert!(error(ScriptError::NotFound).is_ok());
    }
}
