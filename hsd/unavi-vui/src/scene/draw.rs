//! The handful of edits every VUI surface shares.

use wired_guest::math::{
    Color,
    Quat,
    Transform,
    Vec3,
};

use crate::{
    mesh::MeshData,
    view::Style,
    wired::scene::properties::{
        AlphaMode,
        Material,
        Property,
        Topology,
        VertexAttribute,
        VertexStream,
    },
};

pub const fn placed(translation: Vec3, scale: f32) -> Transform {
    Transform {
        translation,
        rotation: Quat::IDENTITY,
        scale: Vec3::splat(scale),
    }
}

/// Like [`placed`], but shifted so a fitted icon's measured centre sits on the
/// origin before the turn: what a spinning, shell-fit icon wears.
pub fn fitted(center: Vec3, scale: f32, rotation: Quat) -> Transform {
    let shifted = rotation * (center * scale);
    Transform {
        translation: -shifted,
        rotation,
        scale: Vec3::splat(scale),
    }
}

/// Scale zero rather than a visibility flag: nothing in `wired:scene` hides a
/// prim, and a body drawn at no size costs no draw call.
pub const fn hidden() -> Transform {
    placed(Vec3::ZERO, 0.0)
}

/// Every edit writing a mesh's topology and its three streams, queued
/// together so a body's geometry costs one batch rather than five.
pub fn mesh(data: &MeshData) -> [Property; 5] {
    [
        Property::MeshTopology(Topology::TriangleList),
        Property::MeshVertices(VertexStream {
            attribute: VertexAttribute::Position,
            values:    data.positions.clone(),
        }),
        Property::MeshVertices(VertexStream {
            attribute: VertexAttribute::Normal,
            values:    data.normals.clone(),
        }),
        Property::MeshVertices(VertexStream {
            attribute: VertexAttribute::Uv0,
            values:    data.uvs.clone(),
        }),
        Property::MeshIndices(data.indices.clone()),
    ]
}

pub const fn with_alpha(color: Color, a: f32) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a,
    }
}

pub const fn scaled(color: Color, factor: f32) -> Color {
    Color {
        r: color.r * factor,
        g: color.g * factor,
        b: color.b * factor,
        a: 1.0,
    }
}

/// A container pip is see-through, like the mote it stands for.
pub const fn pip(style: Style, nested: bool) -> Material {
    Material {
        alpha_cutoff: None,
        alpha_mode:   Some(if nested {
            AlphaMode::Blend
        } else {
            AlphaMode::Opaque
        }),
        base_color:   Some(with_alpha(style.color, if nested { 0.35 } else { 1.0 })),
        double_sided: Some(nested),
        emissive:     Some(scaled(style.color, style.emissive * 1.6)),
        metallic:     None,
        roughness:    None,
    }
}
