//! The handful of prim writes every VUI surface shares.

use wired_prelude::prelude::*;

use crate::{
    mesh::MeshData,
    view::Style,
    wired::scene::types::{
        AlphaMode,
        Material,
        Prim,
        Xform,
    },
};

pub const fn placed(translation: Vec3, scale: f32) -> Xform {
    Xform {
        translation,
        rotation: Quat::IDENTITY,
        scale: Vec3::splat(scale),
    }
}

/// Like [`placed`], but shifted so a fitted icon's measured centre sits on the
/// origin before the turn: what a spinning, shell-fit icon wears.
pub fn fitted(center: Vec3, scale: f32, rotation: Quat) -> Xform {
    let shifted = rotation * (center * scale);
    Xform {
        translation: -shifted,
        rotation,
        scale: Vec3::splat(scale),
    }
}

/// Scale zero rather than a visibility flag: nothing in `wired:scene` hides a
/// prim, and a body drawn at no size costs no draw call.
pub const fn hidden() -> Xform {
    placed(Vec3::ZERO, 0.0)
}

/// Every stream costs a blob upload whatever its size, so this is four per
/// body, paid once when a slot is first built.
pub fn mesh(prim: &Prim, data: &MeshData) -> anyhow::Result<()> {
    prim.set_mesh_stream("POSITION", Some(&data.positions))?;
    prim.set_mesh_stream("NORMAL", Some(&data.normals))?;
    prim.set_mesh_stream("UV_0", Some(&data.uvs))?;
    prim.set_mesh_indices_u32(Some(&data.indices))?;
    Ok(())
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
