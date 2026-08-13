//! What a mote *is*, drawn inside its shell.
//!
//! A slot recognised by silhouette is one that can be found without being
//! read, which a label alone does not give. Authored to fit a unit sphere,
//! because nothing can measure them — `wired:scene` has no bounds query — so
//! the surface scales them to whatever the mote is drawn at.
//!
//! Flat rather than modelled. These hang inside a translucent shell where
//! depth cues fight each other, and a silhouette is what survives the
//! periphery.

use wired_prelude::prelude::*;

use crate::wired::scene::{
    api::self_document,
    types::{
        Material,
        Prim,
        Xform,
    },
};

/// Half-extent of a glyph in the unit sphere the surface scales it into.
/// Under half, so the shell still reads as a shell around it.
const R: f32 = 0.42;

/// (positions, normals, indices).
type MeshData = (Vec<f32>, Vec<f32>, Vec<u32>);

/// Builds the glyph as a prim in this script's own document.
///
/// A prim nothing has parented is a root of its document and would stand at
/// the origin at full size, so it is hidden on the way out; the surface places
/// it once it has a mote to sit in.
pub fn build(mesh: &MeshData, color: Color) -> anyhow::Result<Prim> {
    let prim = self_document()?.create_prim()?;
    prim.set_mesh_stream("POSITION", Some(&mesh.0))?;
    prim.set_mesh_stream("NORMAL", Some(&mesh.1))?;
    prim.set_mesh_indices_u32(Some(&mesh.2))?;
    prim.set_material(Some(Material {
        alpha_cutoff: None,
        alpha_mode:   None,
        base_color:   Some(color),
        // Lit rather than lit-by-the-room: a mote hangs in mid-air, where
        // there is nothing to bounce light off.
        emissive:     Some(Color {
            r: color.r * 0.45,
            g: color.g * 0.45,
            b: color.b * 0.45,
            a: 1.0,
        }),
        double_sided: Some(true),
        metallic:     Some(0.0),
        roughness:    Some(0.6),
    }))?;
    prim.set_xform(Some(Xform {
        translation: Vec3::ZERO,
        rotation:    Quat::IDENTITY,
        scale:       Vec3::ZERO,
    }))?;
    Ok(prim)
}

/// A house: return, the fixed point.
#[must_use]
pub fn home() -> MeshData {
    let w = R * 0.85;
    tris(
        &[
            [0.0, R],
            [-R, R * 0.2],
            [R, R * 0.2],
            [-w, R * 0.2],
            [w, R * 0.2],
            [-w, -R],
            [w, -R],
        ],
        &[0, 1, 2, 3, 5, 4, 4, 5, 6],
    )
}

/// A map pin: outward, the world.
#[must_use]
pub fn places() -> MeshData {
    let head_r = R * 0.62;
    let head_cy = R * 0.32;
    let segments = 10;

    let mut positions = vec![0.0, head_cy, 0.0];
    let mut normals = vec![0.0, 0.0, 1.0];
    for i in 0..=segments {
        let angle = i as f32 / segments as f32 * std::f32::consts::TAU;
        positions.extend_from_slice(&[
            head_r * angle.cos(),
            head_r.mul_add(angle.sin(), head_cy),
            0.0,
        ]);
        normals.extend_from_slice(&[0.0, 0.0, 1.0]);
    }
    let mut indices = Vec::with_capacity(3 * segments);
    for i in 0..segments as u32 {
        indices.extend_from_slice(&[0, 1 + i, 2 + i]);
    }

    let tail_w = head_r * 0.85;
    let tail_base_y = head_r.mul_add(-0.55, head_cy);
    let base = (positions.len() / 3) as u32;
    positions.extend_from_slice(&[
        -tail_w,
        tail_base_y,
        0.0,
        tail_w,
        tail_base_y,
        0.0,
        0.0,
        -R,
        0.0,
    ]);
    normals.extend_from_slice(&[0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0]);
    indices.extend_from_slice(&[base, base + 1, base + 2]);

    (positions, normals, indices)
}

/// A cog: the tools.
#[must_use]
pub fn tools() -> MeshData {
    let teeth = 8;
    let valley = R * 0.68;
    let points = (0..teeth * 2)
        .map(|k| {
            let angle = k as f32 * std::f32::consts::PI / teeth as f32;
            let radius = if k % 2 == 0 { R } else { valley };
            [radius * angle.cos(), radius * angle.sin()]
        })
        .collect::<Vec<_>>();
    fan(&points)
}

/// A diamond: one tool among them.
#[must_use]
pub fn tool() -> MeshData {
    let r = R * 0.85;
    tris(
        &[[0.0, r], [r, 0.0], [0.0, -r], [-r, 0.0]],
        &[0, 3, 1, 1, 3, 2],
    )
}

fn fan(points: &[[f32; 2]]) -> MeshData {
    let mut positions = vec![0.0, 0.0, 0.0];
    let mut normals = vec![0.0, 0.0, 1.0];
    for point in points {
        positions.extend_from_slice(&[point[0], point[1], 0.0]);
        normals.extend_from_slice(&[0.0, 0.0, 1.0]);
    }
    let count = points.len() as u32;
    let mut indices = Vec::with_capacity(3 * points.len());
    for k in 0..count {
        indices.extend_from_slice(&[0, k + 1, (k + 1) % count + 1]);
    }
    (positions, normals, indices)
}

fn tris(verts: &[[f32; 2]], idx: &[u32]) -> MeshData {
    let mut positions = Vec::with_capacity(3 * verts.len());
    let mut normals = Vec::with_capacity(positions.capacity());
    for vert in verts {
        positions.extend_from_slice(&[vert[0], vert[1], 0.0]);
        normals.extend_from_slice(&[0.0, 0.0, 1.0]);
    }
    (positions, normals, idx.to_vec())
}
