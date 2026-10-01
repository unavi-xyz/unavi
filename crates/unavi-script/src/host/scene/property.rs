//! The built-in properties a script reads and writes, and how each maps onto
//! the document's fields.

use std::mem::size_of_val;

use hsd::{
    attributes::{
        collider::{
            self,
            ColliderIndices,
            ColliderKind,
            ColliderVertices,
        },
        gravity_scale::{
            self,
            GravityScaleAttr,
        },
        image::{
            self,
            ImageData,
            ImageSampler,
        },
        material::{
            self,
            BASE_COLOR_TEXTURE,
            BINDING,
            EMISSIVE_TEXTURE,
            METALLIC_ROUGHNESS_TEXTURE,
            MaterialAttr,
            NORMAL_TEXTURE,
            OCCLUSION_TEXTURE,
        },
        mesh::{
            self,
            MeshIndices,
            MeshStream,
            Topology,
        },
        name::{
            self,
            NameAttr,
        },
        parent::{
            self,
            ParentAttr,
        },
        portal::{
            self,
            PortalAttr,
        },
        reference::{
            self,
            ReferenceAttr,
        },
        rigid_body::{
            self,
            RigidBodyAttr,
        },
        script,
        shader::{
            self,
            MAX_TEXTURE_SAMPLES,
        },
        spawn::{
            self,
            SpawnAttr,
        },
        text::{
            self,
            TextAttr,
        },
        xform::{
            self,
            XformAttr,
        },
    },
    bounds::{
        MAX_IMAGE_BYTES,
        MAX_MESH_STREAM_BYTES,
        MAX_NAME_BYTES,
        MAX_TEXT_BYTES,
    },
    id::{
        DocId,
        PrimId,
    },
    prop_name,
    property::{
        Payload,
        Property as _,
        name::PropName,
    },
    state::HsdState,
};
use unavi_physics::finite;

use crate::error::ScriptError;

/// Largest value one custom property may hold.
pub const MAX_CUSTOM_BYTES: usize = 64 * 1024;

/// Every group the host defines. A custom property or relation must live
/// outside them, so a script cannot write raw bytes into a field the host
/// decodes.
const BUILTIN_GROUPS: [&str; 15] = [
    collider::GROUP,
    gravity_scale::GROUP,
    image::GROUP,
    material::GROUP,
    mesh::GROUP,
    name::GROUP,
    parent::GROUP,
    portal::GROUP,
    reference::GROUP,
    rigid_body::GROUP,
    script::GROUP,
    shader::GROUP,
    spawn::GROUP,
    text::GROUP,
    xform::GROUP,
];

/// Parses an application-defined `group/field` name, refusing a bare group or
/// one the host defines.
pub fn custom_name(name: &str) -> Result<PropName, ScriptError> {
    let parsed = name
        .parse::<PropName>()
        .map_err(|_| ScriptError::invalid("a custom name is `group/field`, at most 1 KiB"))?;
    if parsed.field().is_none() {
        return Err(ScriptError::invalid("a custom name is `group/field`"));
    }
    if BUILTIN_GROUPS.contains(&parsed.group()) {
        return Err(ScriptError::invalid(
            "a custom name must not use a group the host defines",
        ));
    }
    Ok(parsed)
}

fn is_custom(name: &PropName) -> bool {
    name.field().is_some() && !BUILTIN_GROUPS.contains(&name.group())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VertexAttribute {
    Position,
    Normal,
    Tangent,
    Color,
    Uv0,
    Uv1,
}

impl VertexAttribute {
    const ALL: [Self; 6] = [
        Self::Position,
        Self::Normal,
        Self::Tangent,
        Self::Color,
        Self::Uv0,
        Self::Uv1,
    ];

    const fn name(self) -> PropName {
        match self {
            Self::Position => prop_name!("mesh/stream:POSITION"),
            Self::Normal => prop_name!("mesh/stream:NORMAL"),
            Self::Tangent => prop_name!("mesh/stream:TANGENT"),
            Self::Color => prop_name!("mesh/stream:COLOR"),
            Self::Uv0 => prop_name!("mesh/stream:UV_0"),
            Self::Uv1 => prop_name!("mesh/stream:UV_1"),
        }
    }

    const fn components(self) -> usize {
        match self {
            Self::Uv0 | Self::Uv1 => 2,
            Self::Position | Self::Normal => 3,
            Self::Tangent | Self::Color => 4,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Relation {
    ShaderBinding,
    BaseColorTexture,
    EmissiveTexture,
    MetallicRoughnessTexture,
    NormalTexture,
    OcclusionTexture,
    ShaderTexture(u8),
    Custom(PropName),
}

impl Relation {
    fn name(&self) -> Result<PropName, ScriptError> {
        Ok(match self {
            Self::ShaderBinding => BINDING,
            Self::BaseColorTexture => BASE_COLOR_TEXTURE,
            Self::EmissiveTexture => EMISSIVE_TEXTURE,
            Self::MetallicRoughnessTexture => METALLIC_ROUGHNESS_TEXTURE,
            Self::NormalTexture => NORMAL_TEXTURE,
            Self::OcclusionTexture => OCCLUSION_TEXTURE,
            Self::ShaderTexture(slot) => shader::texture(*slot)
                .ok_or_else(|| ScriptError::invalid("a shader texture slot is 0 to 3"))?,
            Self::Custom(name) => name.clone(),
        })
    }

    fn from_name(name: &PropName) -> Option<Self> {
        let builtin = [
            (BINDING, Self::ShaderBinding),
            (BASE_COLOR_TEXTURE, Self::BaseColorTexture),
            (EMISSIVE_TEXTURE, Self::EmissiveTexture),
            (METALLIC_ROUGHNESS_TEXTURE, Self::MetallicRoughnessTexture),
            (NORMAL_TEXTURE, Self::NormalTexture),
            (OCCLUSION_TEXTURE, Self::OcclusionTexture),
        ];
        if let Some((_, relation)) = builtin.into_iter().find(|(n, _)| n == name) {
            return Some(relation);
        }
        if let Some(slot) = (0..MAX_TEXTURE_SAMPLES as u8)
            .find(|slot| shader::texture(*slot).as_ref() == Some(name))
        {
            return Some(Self::ShaderTexture(slot));
        }
        is_custom(name).then(|| Self::Custom(name.clone()))
    }
}

/// Names a property without its value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyKey {
    Parent,
    Name,
    Transform,
    Reference,
    MeshTopology,
    MeshVertices(VertexAttribute),
    MeshIndices,
    Material,
    ImageSampler,
    ImageData,
    Text,
    Collider,
    ColliderVertices,
    ColliderIndices,
    RigidBody,
    GravityScale,
    Portal,
    Spawn,
    Relation(Relation),
    Custom(PropName),
}

impl PropertyKey {
    /// The document field this key names.
    pub fn name(&self) -> Result<PropName, ScriptError> {
        Ok(match self {
            Self::Parent => ParentAttr::NAME,
            Self::Name => NameAttr::NAME,
            Self::Transform => XformAttr::NAME,
            Self::Reference => ReferenceAttr::NAME,
            Self::MeshTopology => Topology::NAME,
            Self::MeshVertices(attribute) => attribute.name(),
            Self::MeshIndices => MeshIndices::NAME,
            Self::Material => MaterialAttr::NAME,
            Self::ImageSampler => ImageSampler::NAME,
            Self::ImageData => ImageData::NAME,
            Self::Text => TextAttr::NAME,
            Self::Collider => ColliderKind::NAME,
            Self::ColliderVertices => ColliderVertices::NAME,
            Self::ColliderIndices => ColliderIndices::NAME,
            Self::RigidBody => RigidBodyAttr::NAME,
            Self::GravityScale => GravityScaleAttr::NAME,
            Self::Portal => PortalAttr::NAME,
            Self::Spawn => SpawnAttr::NAME,
            Self::Relation(relation) => relation.name()?,
            Self::Custom(name) => name.clone(),
        })
    }

    /// The key for a document field, `None` for one scripts do not see.
    fn from_name(name: &PropName, is_relation: bool) -> Option<Self> {
        if is_relation {
            return Relation::from_name(name).map(Self::Relation);
        }
        let fixed = [
            (NameAttr::NAME, Self::Name),
            (XformAttr::NAME, Self::Transform),
            (ReferenceAttr::NAME, Self::Reference),
            (Topology::NAME, Self::MeshTopology),
            (MeshIndices::NAME, Self::MeshIndices),
            (MaterialAttr::NAME, Self::Material),
            (ImageSampler::NAME, Self::ImageSampler),
            (ImageData::NAME, Self::ImageData),
            (TextAttr::NAME, Self::Text),
            (ColliderKind::NAME, Self::Collider),
            (ColliderVertices::NAME, Self::ColliderVertices),
            (ColliderIndices::NAME, Self::ColliderIndices),
            (RigidBodyAttr::NAME, Self::RigidBody),
            (GravityScaleAttr::NAME, Self::GravityScale),
            (PortalAttr::NAME, Self::Portal),
            (SpawnAttr::NAME, Self::Spawn),
        ];
        if let Some((_, key)) = fixed.into_iter().find(|(n, _)| n == name) {
            return Some(key);
        }
        if let Some(attribute) = VertexAttribute::ALL
            .into_iter()
            .find(|attribute| attribute.name() == *name)
        {
            return Some(Self::MeshVertices(attribute));
        }
        is_custom(name).then(|| Self::Custom(name.clone()))
    }

    /// Every key with a value on `prim`.
    pub fn all(state: &HsdState, prim: PrimId) -> Vec<Self> {
        let Some(resolved) = state.get(prim) else {
            return Vec::new();
        };
        let mut keys = Vec::new();
        if state.is_in_scene(prim) {
            keys.push(Self::Parent);
        }
        keys.extend(
            resolved.properties().filter_map(|(name, value)| {
                Self::from_name(name, value.as_relationship().is_some())
            }),
        );
        keys
    }

    /// The composed value of this key on `prim`. A stored value that does not
    /// decode reads as absent.
    pub fn read(&self, state: &HsdState, prim: PrimId) -> Option<Property> {
        fn attr<A: hsd::property::Property>(state: &HsdState, prim: PrimId) -> Option<A> {
            state.attribute::<A>(prim)?.ok()
        }

        Some(match self {
            Self::Parent => {
                if !state.is_in_scene(prim) {
                    return None;
                }
                Property::Parent(state.parent(prim))
            }
            Self::Name => Property::Name(attr::<NameAttr>(state, prim)?.0),
            Self::Transform => Property::Transform(attr(state, prim)?),
            Self::Reference => Property::Reference(attr::<ReferenceAttr>(state, prim)?.0),
            Self::MeshTopology => Property::MeshTopology(attr(state, prim)?),
            Self::MeshVertices(attribute) => {
                let stream = state.payload::<MeshStream>(prim, &attribute.name())?.ok()?;
                Property::MeshVertices(*attribute, f32s(&stream.0))
            }
            Self::MeshIndices => Property::MeshIndices(u32s(&attr::<MeshIndices>(state, prim)?.0)),
            Self::Material => Property::Material(attr(state, prim)?),
            Self::ImageSampler => Property::ImageSampler(attr(state, prim)?),
            Self::ImageData => Property::ImageData(attr::<ImageData>(state, prim)?.0),
            Self::Text => Property::Text(attr(state, prim)?),
            Self::Collider => Property::Collider(attr(state, prim)?),
            Self::ColliderVertices => {
                Property::ColliderVertices(f32s(&attr::<ColliderVertices>(state, prim)?.0))
            }
            Self::ColliderIndices => {
                Property::ColliderIndices(u32s(&attr::<ColliderIndices>(state, prim)?.0))
            }
            Self::RigidBody => Property::RigidBody(attr(state, prim)?),
            Self::GravityScale => {
                Property::GravityScale(attr::<GravityScaleAttr>(state, prim)?.scale as f32)
            }
            Self::Portal => Property::Portal(attr(state, prim)?),
            Self::Spawn => Property::Spawn(attr(state, prim)?),
            Self::Relation(relation) => {
                let target = state.relationship(prim, &relation.name().ok()?)?;
                Property::Relation(relation.clone(), target)
            }
            Self::Custom(name) => {
                let value = state.get(prim)?.property(name)?.as_attribute()?;
                Property::Custom(name.clone(), value.to_vec())
            }
        })
    }
}

/// A property and its value.
#[derive(Clone, Debug, PartialEq)]
pub enum Property {
    Parent(Option<PrimId>),
    Name(String),
    Transform(XformAttr),
    Reference(DocId),
    MeshTopology(Topology),
    MeshVertices(VertexAttribute, Vec<f32>),
    MeshIndices(Vec<u32>),
    Material(MaterialAttr),
    ImageSampler(ImageSampler),
    ImageData(Vec<u8>),
    Text(TextAttr),
    Collider(ColliderKind),
    ColliderVertices(Vec<f32>),
    ColliderIndices(Vec<u32>),
    RigidBody(RigidBodyAttr),
    GravityScale(f32),
    Portal(PortalAttr),
    Spawn(SpawnAttr),
    Relation(Relation, PrimId),
    Custom(PropName, Vec<u8>),
}

impl Property {
    #[must_use]
    pub fn key(&self) -> PropertyKey {
        match self {
            Self::Parent(_) => PropertyKey::Parent,
            Self::Name(_) => PropertyKey::Name,
            Self::Transform(_) => PropertyKey::Transform,
            Self::Reference(_) => PropertyKey::Reference,
            Self::MeshTopology(_) => PropertyKey::MeshTopology,
            Self::MeshVertices(attribute, _) => PropertyKey::MeshVertices(*attribute),
            Self::MeshIndices(_) => PropertyKey::MeshIndices,
            Self::Material(_) => PropertyKey::Material,
            Self::ImageSampler(_) => PropertyKey::ImageSampler,
            Self::ImageData(_) => PropertyKey::ImageData,
            Self::Text(_) => PropertyKey::Text,
            Self::Collider(_) => PropertyKey::Collider,
            Self::ColliderVertices(_) => PropertyKey::ColliderVertices,
            Self::ColliderIndices(_) => PropertyKey::ColliderIndices,
            Self::RigidBody(_) => PropertyKey::RigidBody,
            Self::GravityScale(_) => PropertyKey::GravityScale,
            Self::Portal(_) => PropertyKey::Portal,
            Self::Spawn(_) => PropertyKey::Spawn,
            Self::Relation(relation, _) => PropertyKey::Relation(relation.clone()),
            Self::Custom(name, _) => PropertyKey::Custom(name.clone()),
        }
    }

    /// Whether writing this spends the upload budget.
    #[must_use]
    pub const fn is_upload(&self) -> bool {
        matches!(
            self,
            Self::MeshVertices(..)
                | Self::MeshIndices(_)
                | Self::ImageData(_)
                | Self::ColliderVertices(_)
                | Self::ColliderIndices(_)
        )
    }

    /// Refuses a value no layer may hold: too large, or not finite where the
    /// engine needs a number.
    pub fn check(&self) -> Result<(), ScriptError> {
        let ok = match self {
            Self::Parent(_)
            | Self::Reference(_)
            | Self::MeshTopology(_)
            | Self::ImageSampler(_)
            | Self::Relation(..) => true,
            Self::Name(name) => name.len() <= MAX_NAME_BYTES,
            Self::Transform(xform) => {
                finite::vec3(xform.translation).is_some()
                    && finite::quat(xform.rotation).is_some()
                    && finite::vec3(xform.scale).is_some()
            }
            Self::MeshVertices(attribute, values) => {
                size_of_val(values.as_slice()) <= MAX_MESH_STREAM_BYTES
                    && values.len().is_multiple_of(attribute.components())
            }
            Self::MeshIndices(values) | Self::ColliderIndices(values) => {
                size_of_val(values.as_slice()) <= MAX_MESH_STREAM_BYTES
            }
            Self::ColliderVertices(values) => {
                size_of_val(values.as_slice()) <= MAX_MESH_STREAM_BYTES
                    && values.len().is_multiple_of(3)
            }
            Self::ImageData(bytes) => bytes.len() <= MAX_IMAGE_BYTES,
            Self::Text(text) => text.value.len() <= MAX_TEXT_BYTES,
            Self::Collider(kind) => match *kind {
                ColliderKind::Capsule { height, radius }
                | ColliderKind::Cylinder { height, radius } => nonneg64(height) && nonneg64(radius),
                ColliderKind::Cuboid { x, y, z } => nonneg64(x) && nonneg64(y) && nonneg64(z),
                ColliderKind::Sphere(radius) => nonneg64(radius),
                ColliderKind::ConvexHull | ColliderKind::Trimesh => true,
            },
            Self::RigidBody(body) => [
                body.mass,
                body.friction,
                body.restitution,
                body.linear_damping,
                body.angular_damping,
            ]
            .into_iter()
            .flatten()
            .all(nonneg64),
            Self::GravityScale(scale) => scale.is_finite(),
            Self::Portal(portal) => nonneg64(portal.size_x) && nonneg64(portal.size_y),
            Self::Spawn(spawn) => nonneg64(spawn.radius),
            Self::Material(material) => {
                [material.alpha_cutoff, material.metallic, material.roughness]
                    .into_iter()
                    .flatten()
                    .all(f64::is_finite)
            }
            Self::Custom(_, bytes) => bytes.len() <= MAX_CUSTOM_BYTES,
        };
        if ok {
            Ok(())
        } else {
            Err(ScriptError::invalid(self.refusal()))
        }
    }

    const fn refusal(&self) -> &'static str {
        match self {
            Self::Name(_) => "a name is at most 1 KiB",
            Self::Transform(_) => "a transform must be finite",
            Self::MeshVertices(..) => {
                "a vertex stream is at most 4 MiB, a whole number of vertices"
            }
            Self::MeshIndices(_) | Self::ColliderIndices(_) => "an index buffer is at most 4 MiB",
            Self::ColliderVertices(_) => {
                "collider vertices are at most 4 MiB, a whole number of positions"
            }
            Self::ImageData(_) => "image data is at most 16 MiB",
            Self::Text(_) => "text is at most 4 KiB",
            Self::Custom(..) => "a custom value is at most 64 KiB",
            _ => "a size or scalar must be finite and not negative",
        }
    }

    /// The field's stored bytes, or `None` for a relation, which is not
    /// bytes but a link.
    pub fn payload(&self) -> Result<Option<Vec<u8>>, ScriptError> {
        let encoded = match self {
            Self::Relation(..) => return Ok(None),
            Self::Custom(_, bytes) => return Ok(Some(bytes.clone())),
            Self::Parent(parent) => parent.map_or(ParentAttr::Root, ParentAttr::Prim).encode(),
            Self::Name(name) => NameAttr(name.clone()).encode(),
            Self::Transform(xform) => xform.encode(),
            Self::Reference(doc) => ReferenceAttr(*doc).encode(),
            Self::MeshTopology(topology) => topology.encode(),
            Self::MeshVertices(_, values) => MeshStream(f32_bytes(values)).encode(),
            Self::MeshIndices(values) => MeshIndices(u32_bytes(values)).encode(),
            Self::Material(material) => material.encode(),
            Self::ImageSampler(sampler) => sampler.encode(),
            Self::ImageData(bytes) => ImageData(bytes.clone()).encode(),
            Self::Text(text) => text.encode(),
            Self::Collider(kind) => kind.encode(),
            Self::ColliderVertices(values) => ColliderVertices(f32_bytes(values)).encode(),
            Self::ColliderIndices(values) => ColliderIndices(u32_bytes(values)).encode(),
            Self::RigidBody(body) => body.encode(),
            Self::GravityScale(scale) => GravityScaleAttr {
                scale: f64::from(*scale),
            }
            .encode(),
            Self::Portal(portal) => portal.encode(),
            Self::Spawn(spawn) => spawn.encode(),
        };
        encoded.map(Some).map_err(ScriptError::internal)
    }
}

fn nonneg64(v: f64) -> bool {
    v.is_finite() && v >= 0.0
}

fn f32_bytes(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

fn u32_bytes(values: &[u32]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_le_bytes()).collect()
}

fn f32s(bytes: &[u8]) -> Vec<f32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| f32::from_le_bytes(*chunk))
        .collect()
}

fn u32s(bytes: &[u8]) -> Vec<u32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| u32::from_le_bytes(*chunk))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_custom_name_cannot_reach_a_builtin_field() {
        assert!(custom_name("xform/value").is_err());
        assert!(custom_name("material/binding").is_err());
        assert!(custom_name("game").is_err(), "a bare group has no field");
        assert!(custom_name("game/score").is_ok());
    }

    #[test]
    fn every_key_round_trips_through_its_field_name() {
        let keys = [
            PropertyKey::Name,
            PropertyKey::Transform,
            PropertyKey::MeshVertices(VertexAttribute::Uv1),
            PropertyKey::Collider,
            PropertyKey::Custom(custom_name("game/score").expect("valid")),
        ];
        for key in keys {
            let name = key.name().expect("named");
            assert_eq!(PropertyKey::from_name(&name, false), Some(key));
        }
        let relation = PropertyKey::Relation(Relation::ShaderTexture(2));
        assert_eq!(
            PropertyKey::from_name(&relation.name().expect("named"), true),
            Some(relation)
        );
    }

    #[test]
    fn a_written_property_reads_back() {
        let mut state = HsdState::new();
        let prim = state.create_prim(None);
        let written = Property::MeshVertices(VertexAttribute::Position, vec![1.0, 2.0, 3.0]);
        let payload = written.payload().expect("encode").expect("bytes");
        state
            .set_property(
                prim,
                &written.key().name().expect("named"),
                hsd::property::value::Value::Attribute(payload.into()),
            )
            .expect("write");
        assert_eq!(written.key().read(&state, prim), Some(written));
    }

    #[test]
    fn a_ragged_vertex_stream_is_refused() {
        assert!(
            Property::MeshVertices(VertexAttribute::Position, vec![0.0; 4])
                .check()
                .is_err()
        );
    }
}
