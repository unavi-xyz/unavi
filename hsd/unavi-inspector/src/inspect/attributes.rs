//! A prim's attributes as components: one card per thing a prim states (where
//! it is, its mesh, its body), each with a summary to show folded and the
//! fields to show open. Pure, over the keys and values read from the prim.

use wired_guest::math::{
    Color,
    Transform,
};

use super::format::words;
use crate::wired::scene::properties::{
    AlphaMode,
    Collider,
    Material,
    Property,
    PropertyKey,
    RigidBody,
    RigidBodyKind,
    Topology,
    VertexAttribute,
};

/// One component of a prim.
pub struct Card {
    pub name:    String,
    /// What it says folded.
    pub summary: String,
    /// The key whose layer the card is said to come from.
    pub key:     PropertyKey,
    /// Label and value of each field, shown open.
    pub fields:  Vec<(String, String)>,
}

impl Card {
    fn new(name: &str, summary: String, key: &PropertyKey) -> Self {
        Self {
            name: name.to_owned(),
            summary,
            key: key.clone(),
            fields: Vec::new(),
        }
    }

    fn field(mut self, label: &str, value: impl Into<String>) -> Self {
        self.fields.push((label.to_owned(), value.into()));
        self
    }
}

/// The name of the card a transform is on, which edits in place.
pub const TRANSFORM: &str = "Transform";

/// The cards of a prim stating `stated`, in a fixed order: placement, look,
/// physics, then the rest. The name and parent are left out, since the
/// slate's heading and the document show them.
#[must_use]
pub fn cards(stated: &[(PropertyKey, Property)]) -> Vec<Card> {
    let mut out = Vec::new();
    for (key, property) in stated {
        if let Property::Transform(at) = property {
            out.push(transform(at, key));
        }
    }
    if let Some(card) = mesh(stated) {
        out.push(card);
    }
    for (key, property) in stated {
        if let Property::Material(material) = property {
            out.push(material_card(material, key));
        }
    }
    if let Some(card) = collider(stated) {
        out.push(card);
    }
    if let Some(card) = body(stated) {
        out.push(card);
    }
    out.extend(
        stated
            .iter()
            .filter_map(|(key, property)| other(key, property)),
    );
    out
}

fn transform(at: &Transform, key: &PropertyKey) -> Card {
    let t = at.translation;
    let position = format!("{:.2}  {:.2}  {:.2}", t.x, t.y, t.z);
    let rotation = glam::Vec4::from(glam::Quat::from(at.rotation))
        .try_normalize()
        .map(glam::Quat::from_vec4)
        .map_or_else(
            || "invalid".to_owned(),
            |rotation| {
                let (yaw, pitch, roll) = rotation.to_euler(glam::EulerRot::YXZ);
                format!(
                    "{:.0}°  {:.0}°  {:.0}°",
                    degrees(yaw),
                    degrees(pitch),
                    degrees(roll)
                )
            },
        );
    let s = at.scale;
    let uniform = (s.x - s.y).abs() < 1.0e-4 && (s.y - s.z).abs() < 1.0e-4;
    let scale = if uniform {
        format!("{:.2}", s.x)
    } else {
        format!("{:.2}  {:.2}  {:.2}", s.x, s.y, s.z)
    };
    Card::new(TRANSFORM, position.clone(), key)
        .field("Position", position)
        .field("Rotation", rotation)
        .field("Scale", scale)
}

/// Degrees, without a negative zero.
fn degrees(radians: f32) -> f32 {
    let degrees = radians.to_degrees();
    if degrees.abs() < 0.5 { 0.0 } else { degrees }
}

fn mesh(stated: &[(PropertyKey, Property)]) -> Option<Card> {
    let mut topology = None;
    let mut position = None;
    let mut streams = Vec::new();
    let mut indices = None;
    for (key, property) in stated {
        match property {
            Property::MeshTopology(kind) => topology = Some(*kind),
            Property::MeshVertices(stream) => {
                streams.push(words(&format!("{:?}", stream.attribute)));
                if stream.attribute == VertexAttribute::Position {
                    position = Some((key, stream.values.len() / 3));
                }
            }
            Property::MeshIndices(list) => indices = Some(list.len()),
            _ => {}
        }
    }
    let (key, vertices) = position?;
    let triangles = match (topology, indices) {
        (Some(Topology::TriangleList) | None, Some(indices)) => Some(indices / 3),
        (Some(Topology::TriangleList) | None, None) => Some(vertices / 3),
        _ => None,
    };
    let summary = triangles.map_or_else(
        || format!("{vertices} vertices"),
        |triangles| format!("{triangles} triangles"),
    );
    let mut card = Card::new("Mesh", summary, key).field("Vertices", vertices.to_string());
    if let Some(kind) = topology {
        card = card.field("Topology", topology_name(kind));
    }
    Some(card.field("Streams", streams.join(", ")))
}

fn material_card(material: &Material, key: &PropertyKey) -> Card {
    let base = material
        .base_color
        .map_or_else(|| "default".to_owned(), hex);
    let mut card = Card::new("Material", base.clone(), key).field("Colour", base);
    if let Some(mode) = material.alpha_mode {
        card = card.field("Alpha", alpha(mode));
    }
    if let Some(metallic) = material.metallic {
        card = card.field("Metallic", format!("{metallic:.2}"));
    }
    if let Some(roughness) = material.roughness {
        card = card.field("Roughness", format!("{roughness:.2}"));
    }
    if let Some(glow) = material.emissive {
        card = card.field("Glow", hex(glow));
    }
    card
}

fn collider(stated: &[(PropertyKey, Property)]) -> Option<Card> {
    let (key, shape) = stated.iter().find_map(|(key, property)| match property {
        Property::Collider(shape) => Some((key, shape)),
        _ => None,
    })?;
    let triangles = stated.iter().find_map(|(_, property)| match property {
        Property::ColliderIndices(indices) => Some(indices.len() / 3),
        _ => None,
    });
    let (kind, size) = match shape {
        Collider::Cuboid(size) => (
            "box",
            format!("{:.2} × {:.2} × {:.2}", size.x, size.y, size.z),
        ),
        Collider::Sphere(radius) => ("sphere", format!("r {radius:.2}")),
        Collider::Capsule(capsule) => (
            "capsule",
            format!("r {:.2}  h {:.2}", capsule.radius, capsule.height),
        ),
        Collider::Cylinder(cylinder) => (
            "cylinder",
            format!("r {:.2}  h {:.2}", cylinder.radius, cylinder.height),
        ),
        Collider::ConvexHull => ("hull", String::new()),
        Collider::Trimesh => (
            "mesh",
            triangles.map_or_else(String::new, |count| format!("{count} triangles")),
        ),
    };
    let mut card = Card::new("Collider", kind.to_owned(), key).field("Shape", kind);
    if !size.is_empty() {
        card = card.field("Size", size);
    }
    Some(card)
}

fn body(stated: &[(PropertyKey, Property)]) -> Option<Card> {
    let (key, body): (&PropertyKey, &RigidBody) =
        stated.iter().find_map(|(key, property)| match property {
            Property::RigidBody(body) => Some((key, body)),
            _ => None,
        })?;
    let kind = match body.kind {
        RigidBodyKind::Dynamic => "dynamic",
        RigidBodyKind::Kinematic => "kinematic",
        RigidBodyKind::Static => "static",
    };
    let mut card = Card::new("Body", kind.to_owned(), key).field("Kind", kind);
    let numbers = [
        ("Mass", body.mass),
        ("Friction", body.friction),
        ("Bounce", body.restitution),
        ("Drag", body.linear_damping),
        ("Spin drag", body.angular_damping),
    ];
    for (label, value) in numbers {
        if let Some(value) = value {
            card = card.field(label, format!("{value:.2}"));
        }
    }
    let gravity = stated.iter().find_map(|(_, property)| match property {
        Property::GravityScale(scale) => Some(*scale),
        _ => None,
    });
    if let Some(gravity) = gravity {
        card = card.field("Gravity", format!("{gravity:.2}"));
    }
    Some(card)
}

/// A card for a key the cards above do not cover, or `None` for one they
/// do or one the slate shows elsewhere.
fn other(key: &PropertyKey, property: &Property) -> Option<Card> {
    Some(match property {
        Property::Parent(_)
        | Property::Name(_)
        | Property::Transform(_)
        | Property::MeshTopology(_)
        | Property::MeshVertices(_)
        | Property::MeshIndices(_)
        | Property::Material(_)
        | Property::ImageSampler(_)
        | Property::Collider(_)
        | Property::ColliderVertices(_)
        | Property::ColliderIndices(_)
        | Property::RigidBody(_)
        | Property::GravityScale(_) => return None,
        Property::ImageData(bytes) => Card::new("Image", size_of(bytes.len()), key),
        Property::Text(text) => {
            let first = text.value.lines().next().unwrap_or_default();
            Card::new("Text", format!("“{first}”"), key).field("Text", text.value.clone())
        }
        Property::Reference(doc) => Card::new("Reference", wired_guest::color::short_id(*doc), key),
        Property::Sky(color) => Card::new("Sky", hex(*color), key),
        Property::Portal(portal) => Card::new(
            "Portal",
            format!("{:.2} × {:.2}", portal.width, portal.height),
            key,
        )
        .field(
            "Leads to",
            portal.destination.as_ref().map_or_else(
                || "nowhere yet".to_owned(),
                |to| wired_guest::color::short_id(to.space),
            ),
        ),
        Property::Spawn(spawn) => Card::new("Spawn", format!("r {:.2}", spawn.radius), key),
        Property::Relation((relation, _)) => Card::new(
            &capitalised(&words(&format!("{relation:?}"))),
            "linked".to_owned(),
            key,
        ),
        Property::Custom((name, bytes)) if name == wired_guest::LOCKED => {
            Card::new("Locked", custom_value(bytes), key)
        }
        Property::Custom((name, bytes)) => Card::new(name, custom_value(bytes), key),
    })
}

fn capitalised(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().chain(chars).collect()
    })
}

fn custom_value(bytes: &[u8]) -> String {
    match bytes {
        [] => "empty".to_owned(),
        [0] => "off".to_owned(),
        [1] => "on".to_owned(),
        [byte] => byte.to_string(),
        _ => size_of(bytes.len()),
    }
}

const fn alpha(mode: AlphaMode) -> &'static str {
    match mode {
        AlphaMode::Opaque => "opaque",
        AlphaMode::Mask => "cut out",
        AlphaMode::Blend => "see-through",
        AlphaMode::Add => "additive",
        AlphaMode::Multiply => "multiply",
        AlphaMode::Premultiplied => "premultiplied",
    }
}

const fn topology_name(kind: Topology) -> &'static str {
    match kind {
        Topology::PointList => "points",
        Topology::LineList | Topology::LineStrip => "lines",
        Topology::TriangleList => "triangles",
        Topology::TriangleStrip => "triangle strip",
    }
}

fn hex(color: Color) -> String {
    let byte = |channel: f32| (channel.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!(
        "#{:02x}{:02x}{:02x}",
        byte(color.r),
        byte(color.g),
        byte(color.b)
    )
}

fn size_of(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else {
        format!("{} KiB", bytes / 1024)
    }
}

#[cfg(test)]
mod tests {
    use wired_guest::math::{
        Quat,
        Vec3,
    };

    use super::*;
    use crate::wired::scene::properties::VertexStream;

    fn names(stated: &[(PropertyKey, Property)]) -> Vec<String> {
        cards(stated).into_iter().map(|card| card.name).collect()
    }

    fn cube_mesh() -> Vec<(PropertyKey, Property)> {
        let stream = |attribute| VertexStream {
            attribute,
            values: vec![0.0; 24 * 3],
        };
        vec![
            (
                PropertyKey::MeshTopology,
                Property::MeshTopology(Topology::TriangleList),
            ),
            (
                PropertyKey::MeshVertices(VertexAttribute::Position),
                Property::MeshVertices(stream(VertexAttribute::Position)),
            ),
            (
                PropertyKey::MeshVertices(VertexAttribute::Normal),
                Property::MeshVertices(stream(VertexAttribute::Normal)),
            ),
            (PropertyKey::MeshIndices, Property::MeshIndices(vec![0; 36])),
        ]
    }

    #[test]
    fn a_mesh_is_one_card_however_many_keys_it_takes() {
        let cards = cards(&cube_mesh());
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].name, "Mesh");
        assert_eq!(cards[0].summary, "12 triangles");
        assert!(
            cards[0]
                .fields
                .iter()
                .any(|(label, value)| label == "Streams" && value == "position, normal")
        );
    }

    #[test]
    fn a_transform_card_sums_up_where_it_is_and_lists_its_parts() {
        let at = Transform::new(
            Vec3::new(1.0, 2.0, 3.0),
            Quat::from(glam::Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
            Vec3::splat(2.0),
        );
        let cards = cards(&[(PropertyKey::Transform, Property::Transform(at))]);
        assert_eq!(cards[0].summary, "1.00  2.00  3.00");
        let fields: Vec<_> = cards[0]
            .fields
            .iter()
            .map(|(label, value)| (label.as_str(), value.as_str()))
            .collect();
        assert_eq!(
            fields,
            [
                ("Position", "1.00  2.00  3.00"),
                ("Rotation", "90°  0°  0°"),
                ("Scale", "2.00"),
            ]
        );
    }

    #[test]
    fn a_rotation_that_is_no_rotation_reads_as_invalid() {
        let at = Transform::new(
            Vec3::ZERO,
            Quat::from(glam::Quat::from_xyzw(0.0, 0.0, 0.0, 0.0)),
            Vec3::ONE,
        );
        let cards = cards(&[(PropertyKey::Transform, Property::Transform(at))]);
        assert_eq!(cards[0].fields[1].1, "invalid");
    }

    #[test]
    fn placement_comes_before_the_look_and_the_body() {
        let mut stated = vec![(
            PropertyKey::RigidBody,
            Property::RigidBody(RigidBody {
                kind:            RigidBodyKind::Dynamic,
                mass:            Some(2.0),
                friction:        None,
                restitution:     None,
                linear_damping:  None,
                angular_damping: None,
            }),
        )];
        stated.extend(cube_mesh());
        stated.push((
            PropertyKey::Transform,
            Property::Transform(Transform::IDENTITY),
        ));
        stated.push((PropertyKey::Name, Property::Name("Crate".to_owned())));
        assert_eq!(names(&stated), ["Transform", "Mesh", "Body"]);
    }

    #[test]
    fn a_custom_attribute_is_a_card_named_by_it() {
        let cards = cards(&[(
            PropertyKey::Custom("game/score".to_owned()),
            Property::Custom(("game/score".to_owned(), vec![7])),
        )]);
        assert_eq!(cards[0].name, "game/score");
        assert_eq!(cards[0].summary, "7");
    }
}
