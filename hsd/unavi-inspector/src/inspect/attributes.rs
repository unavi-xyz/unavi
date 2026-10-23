//! The Attributes tab's lines: what a prim states, one line per thing a
//! person thinks of (where it is, its mesh, its body) rather than one per
//! stored key. Pure, over the keys and values read from the prim.

use wired_guest::math::{
    Color,
    Transform,
};

use super::format::words;
use crate::wired::scene::properties::{
    AlphaMode,
    Collider,
    Property,
    PropertyKey,
    RigidBody,
    RigidBodyKind,
    Topology,
    VertexAttribute,
};

/// One line of the tab: a label, what it reads, and the key whose layer the
/// line is said to come from.
pub struct Line {
    pub label: String,
    pub value: String,
    pub key:   PropertyKey,
}

impl Line {
    fn new(label: &str, value: String, key: PropertyKey) -> Self {
        Self {
            label: label.to_owned(),
            value,
            key,
        }
    }
}

/// The lines for a prim stating `stated`, in a fixed order: placement, look,
/// physics, then the rest. The name and parent are left out, since the
/// header and the Tree tab show them.
#[must_use]
pub fn lines(stated: &[(PropertyKey, Property)]) -> Vec<Line> {
    let find = |wanted: fn(&Property) -> bool| {
        stated
            .iter()
            .find(|(_, property)| wanted(property))
            .map(|(key, property)| (key.clone(), property))
    };
    let mut out = Vec::new();
    if let Some((key, Property::Transform(at))) = find(|p| matches!(p, Property::Transform(_))) {
        placement(at, &key, &mut out);
    }
    if let Some(line) = mesh(stated) {
        out.push(line);
    }
    if let Some((key, Property::Material(material))) = find(|p| matches!(p, Property::Material(_)))
    {
        let mut parts = vec![
            material
                .base_color
                .map_or_else(|| "default".to_owned(), hex),
        ];
        if let Some(mode) = material.alpha_mode
            && mode != AlphaMode::Opaque
        {
            parts.push(alpha(mode).to_owned());
        }
        if material.emissive.is_some() {
            parts.push("glows".to_owned());
        }
        out.push(Line::new("Material", parts.join(" · "), key));
    }
    if let Some((key, Property::ImageData(bytes))) = find(|p| matches!(p, Property::ImageData(_))) {
        out.push(Line::new("Image", kib(bytes.len()), key));
    }
    if let Some((key, Property::Collider(collider))) = find(|p| matches!(p, Property::Collider(_)))
    {
        let triangles = stated.iter().find_map(|(_, p)| match p {
            Property::ColliderIndices(indices) => Some(indices.len() / 3),
            _ => None,
        });
        out.push(Line::new("Collider", shape(collider, triangles), key));
    }
    if let Some((key, Property::RigidBody(body))) = find(|p| matches!(p, Property::RigidBody(_))) {
        let gravity = stated.iter().find_map(|(_, p)| match p {
            Property::GravityScale(scale) => Some(*scale),
            _ => None,
        });
        out.push(Line::new("Body", body_value(body, gravity), key));
    }
    for (key, property) in stated {
        if let Some(line) = other(key, property) {
            out.push(line);
        }
    }
    out
}

fn placement(at: &Transform, key: &PropertyKey, out: &mut Vec<Line>) {
    let t = at.translation;
    out.push(Line::new(
        "Position",
        format!("{:.2}  {:.2}  {:.2}", t.x, t.y, t.z),
        key.clone(),
    ));
    // Any peer may state a rotation, so it is normalized before it is read.
    let rotation = glam::Vec4::from(glam::Quat::from(at.rotation))
        .try_normalize()
        .map(glam::Quat::from_vec4);
    let value = rotation.map_or_else(
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
    out.push(Line::new("Rotation", value, key.clone()));
    let s = at.scale;
    let uniform = (s.x - s.y).abs() < 1.0e-4 && (s.y - s.z).abs() < 1.0e-4;
    out.push(Line::new(
        "Scale",
        if uniform {
            format!("{:.2}", s.x)
        } else {
            format!("{:.2}  {:.2}  {:.2}", s.x, s.y, s.z)
        },
        key.clone(),
    ));
}

/// Degrees, without a negative zero.
fn degrees(radians: f32) -> f32 {
    let degrees = radians.to_degrees();
    if degrees.abs() < 0.5 { 0.0 } else { degrees }
}

/// The topology, vertex streams and indices as one line.
fn mesh(stated: &[(PropertyKey, Property)]) -> Option<Line> {
    let mut topology = None;
    let mut vertices = None;
    let mut indices = None;
    for (key, property) in stated {
        match property {
            Property::MeshTopology(kind) => topology = Some(*kind),
            Property::MeshVertices(stream) if stream.attribute == VertexAttribute::Position => {
                vertices = Some((key, stream.values.len() / 3));
            }
            Property::MeshIndices(list) => indices = Some(list.len()),
            _ => {}
        }
    }
    let (key, count) = vertices?;
    let value = match (topology, indices) {
        (Some(Topology::TriangleList) | None, Some(indices)) => {
            format!("{count} vertices · {} triangles", indices / 3)
        }
        (Some(Topology::TriangleList) | None, None) => format!("{count} vertices"),
        (Some(kind), _) => format!("{count} vertices · {}", topology_name(kind)),
    };
    Some(Line::new("Mesh", value, key.clone()))
}

/// A line for a key the fixed lines above do not cover, or `None` for one
/// they do or one the slate shows elsewhere.
fn other(key: &PropertyKey, property: &Property) -> Option<Line> {
    let (label, value) = match property {
        Property::Parent(_)
        | Property::Name(_)
        | Property::Transform(_)
        | Property::MeshTopology(_)
        | Property::MeshVertices(_)
        | Property::MeshIndices(_)
        | Property::Material(_)
        | Property::ImageData(_)
        | Property::ImageSampler(_)
        | Property::Collider(_)
        | Property::ColliderVertices(_)
        | Property::ColliderIndices(_)
        | Property::RigidBody(_)
        | Property::GravityScale(_) => return None,
        Property::Text(text) => (
            "Text".to_owned(),
            format!("“{}”", text.value.lines().next().unwrap_or_default()),
        ),
        Property::Reference(doc) => ("Reference".to_owned(), format!("…{:04x}", doc.3 & 0xFFFF)),
        Property::Sky(color) => ("Sky".to_owned(), hex(*color)),
        Property::Portal(portal) => (
            "Portal".to_owned(),
            format!(
                "{:.2} × {:.2} m · {}",
                portal.width,
                portal.height,
                if portal.destination.is_some() {
                    "linked"
                } else {
                    "open"
                }
            ),
        ),
        Property::Spawn(spawn) => ("Spawn".to_owned(), format!("{:.2} m around", spawn.radius)),
        Property::Relation((relation, _)) => (
            capitalised(&words(&format!("{relation:?}"))),
            "linked".to_owned(),
        ),
        Property::Custom((name, bytes)) if name == wired_guest::LOCKED => {
            ("Locked".to_owned(), custom_value(bytes))
        }
        Property::Custom((name, bytes)) => (name.clone(), custom_value(bytes)),
    };
    Some(Line {
        label,
        value,
        key: key.clone(),
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
        _ => format!("{} bytes", bytes.len()),
    }
}

fn shape(collider: &Collider, triangles: Option<usize>) -> String {
    match collider {
        Collider::Cuboid(size) => format!("box {:.2} × {:.2} × {:.2}", size.x, size.y, size.z),
        Collider::Sphere(radius) => format!("sphere r {radius:.2}"),
        Collider::Capsule(capsule) => {
            format!("capsule r {:.2} h {:.2}", capsule.radius, capsule.height)
        }
        Collider::Cylinder(cylinder) => {
            format!("cylinder r {:.2} h {:.2}", cylinder.radius, cylinder.height)
        }
        Collider::ConvexHull => "convex hull".to_owned(),
        Collider::Trimesh => triangles.map_or_else(
            || "mesh".to_owned(),
            |triangles| format!("mesh · {triangles} triangles"),
        ),
    }
}

fn body_value(body: &RigidBody, gravity: Option<f32>) -> String {
    let mut parts = vec![
        match body.kind {
            RigidBodyKind::Dynamic => "dynamic",
            RigidBodyKind::Kinematic => "kinematic",
            RigidBodyKind::Static => "static",
        }
        .to_owned(),
    ];
    if let Some(mass) = body.mass {
        parts.push(format!("{mass:.2} kg"));
    }
    if let Some(gravity) = gravity
        && (gravity - 1.0).abs() > 1.0e-3
    {
        parts.push(format!("gravity {gravity:.2}"));
    }
    parts.join(" · ")
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

fn kib(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} bytes")
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

    fn labels(stated: &[(PropertyKey, Property)]) -> Vec<String> {
        lines(stated).into_iter().map(|line| line.label).collect()
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
            (
                PropertyKey::MeshVertices(VertexAttribute::Uv0),
                Property::MeshVertices(stream(VertexAttribute::Uv0)),
            ),
            (PropertyKey::MeshIndices, Property::MeshIndices(vec![0; 36])),
        ]
    }

    #[test]
    fn a_mesh_reads_as_one_line() {
        let lines = lines(&cube_mesh());
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].label, "Mesh");
        assert_eq!(lines[0].value, "24 vertices · 12 triangles");
    }

    #[test]
    fn a_transform_reads_as_position_rotation_and_scale() {
        let at = Transform::new(
            Vec3::new(1.0, 2.0, 3.0),
            Quat::from(glam::Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
            Vec3::splat(2.0),
        );
        let lines = lines(&[(PropertyKey::Transform, Property::Transform(at))]);
        let read: Vec<_> = lines
            .iter()
            .map(|line| (line.label.as_str(), line.value.as_str()))
            .collect();
        assert_eq!(
            read,
            [
                ("Position", "1.00  2.00  3.00"),
                ("Rotation", "90°  0°  0°"),
                ("Scale", "2.00"),
            ]
        );
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
        assert_eq!(
            labels(&stated),
            ["Position", "Rotation", "Scale", "Mesh", "Body"]
        );
    }

    #[test]
    fn a_rotation_that_is_no_rotation_reads_as_invalid() {
        let at = Transform::new(
            Vec3::ZERO,
            Quat::from(glam::Quat::from_xyzw(0.0, 0.0, 0.0, 0.0)),
            Vec3::ONE,
        );
        let lines = lines(&[(PropertyKey::Transform, Property::Transform(at))]);
        assert_eq!(lines[1].value, "invalid");
    }

    #[test]
    fn a_custom_attribute_is_labelled_by_its_name() {
        let lines = lines(&[(
            PropertyKey::Custom("game/score".to_owned()),
            Property::Custom(("game/score".to_owned(), vec![7])),
        )]);
        assert_eq!(lines[0].label, "game/score");
        assert_eq!(lines[0].value, "7");
    }
}
