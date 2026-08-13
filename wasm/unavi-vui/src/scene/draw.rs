//! The handful of prim writes every VUI surface shares.

use wired_prelude::prelude::*;

use crate::{
    mesh::MeshData,
    mote::Role,
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

/// Scale zero rather than a visibility flag: nothing in `wired:scene` hides a
/// prim, and a body drawn at no size costs no draw call.
pub const fn hidden() -> Xform {
    placed(Vec3::ZERO, 0.0)
}

pub fn mesh(prim: &Prim, data: &MeshData) -> anyhow::Result<()> {
    prim.set_mesh_stream("POSITION", Some(&data.positions))?;
    prim.set_mesh_stream("NORMAL", Some(&data.normals))?;
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

/// A mote's shell. Containers are see-through and commands are solid, which is
/// the difference read before any label is; an item is glass exactly when it
/// holds an icon, so what it is shows through.
pub const fn body(style: Style, role: Role, icon: bool) -> Material {
    let opaque = match role {
        Role::Group { .. } => false,
        Role::Item { .. } => !icon,
        Role::Action | Role::Toggle | Role::Cast | Role::Parent { .. } => true,
    };
    Material {
        alpha_cutoff: None,
        alpha_mode:   Some(if opaque {
            AlphaMode::Opaque
        } else {
            AlphaMode::Blend
        }),
        base_color:   Some(with_alpha(style.color, style.alpha)),
        double_sided: Some(!opaque),
        emissive:     Some(scaled(style.color, style.emissive)),
        metallic:     None,
        roughness:    None,
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
