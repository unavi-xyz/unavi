use std::f32::consts::{
    PI,
    TAU,
};

use crate::wired::scene::types::Prim;

pub const SECTOR_INNER_R: f32 = 0.05;
pub const RING_RADIUS: f32 = 0.16;
pub const SECTOR_GAP_WORLD: f32 = 0.012;
pub const SECTOR_SUBDIVISIONS: usize = 40;
pub const OUTLINE_WIDTH: f32 = 0.006;
pub const OUTLINE_Z: f32 = 0.001;
pub const ICON_R: f32 = (SECTOR_INNER_R + RING_RADIUS) * 0.5;

/// (positions, normals, indices).
pub type MeshData = (Vec<f32>, Vec<f32>, Vec<u32>);

pub fn apply_mesh(prim: &Prim, mesh: &MeshData) {
    prim.set_mesh_stream("POSITION", Some(&mesh.0)).ok();
    prim.set_mesh_stream("NORMAL", Some(&mesh.1)).ok();
    prim.set_mesh_indices_u32(Some(&mesh.2)).ok();
}

fn fan(points: &[[f32; 2]], z: f32) -> MeshData {
    let mut positions = Vec::with_capacity(3 * (points.len() + 1));
    let mut normals = Vec::with_capacity(positions.capacity());
    positions.extend_from_slice(&[0.0, 0.0, z]);
    normals.extend_from_slice(&[0.0, 0.0, 1.0]);
    for p in points {
        positions.extend_from_slice(&[p[0], p[1], z]);
        normals.extend_from_slice(&[0.0, 0.0, 1.0]);
    }
    let mut indices = Vec::with_capacity(3 * points.len());
    for k in 0..points.len() as u32 {
        indices.extend_from_slice(&[0, k + 1, (k + 1) % points.len() as u32 + 1]);
    }
    (positions, normals, indices)
}

fn tris(verts: &[[f32; 2]], idx: &[u32], z: f32) -> MeshData {
    let mut positions = Vec::with_capacity(3 * verts.len());
    let mut normals = Vec::with_capacity(positions.capacity());
    for v in verts {
        positions.extend_from_slice(&[v[0], v[1], z]);
        normals.extend_from_slice(&[0.0, 0.0, 1.0]);
    }
    (positions, normals, idx.to_vec())
}

#[must_use]
pub fn sector_mesh(i: usize, n: usize) -> MeshData {
    annulus(i, n, SECTOR_INNER_R, RING_RADIUS, 0.0)
}

#[must_use]
pub fn outline_mesh(i: usize, n: usize) -> MeshData {
    annulus(i, n, RING_RADIUS, RING_RADIUS + OUTLINE_WIDTH, OUTLINE_Z)
}

fn annulus(i: usize, n: usize, r_inner: f32, r_outer: f32, depth: f32) -> MeshData {
    let half_span = PI / n as f32;
    let center = i as f32 * 2.0 * PI / n as f32;
    let subs = SECTOR_SUBDIVISIONS;

    let mut positions = Vec::with_capacity(3 * 2 * (subs + 1));
    let mut normals = Vec::with_capacity(positions.capacity());
    let mut indices = Vec::with_capacity(6 * subs);

    for r in [r_inner, r_outer] {
        let half_gap = SECTOR_GAP_WORLD / (2.0 * r);
        let start = center - half_span + half_gap;
        let end = center + half_span - half_gap;
        for j in 0..=subs {
            let frac = j as f32 / subs as f32;
            let angle = frac.mul_add(end - start, start);
            positions.extend_from_slice(&[r * angle.cos(), r * angle.sin(), depth]);
            normals.extend_from_slice(&[0.0, 0.0, 1.0]);
        }
    }
    for j in 0..subs as u32 {
        let row = subs as u32 + 1;
        indices.extend_from_slice(&[j, row + j, j + 1, j + 1, row + j, row + j + 1]);
    }
    (positions, normals, indices)
}

const GLYPH: f32 = 0.018;

/// A house silhouette.
#[must_use]
pub fn home_mesh() -> MeshData {
    let w = GLYPH * 0.85;
    let verts = [
        [0.0, GLYPH],
        [-GLYPH, GLYPH * 0.2],
        [GLYPH, GLYPH * 0.2],
        [-w, GLYPH * 0.2],
        [w, GLYPH * 0.2],
        [-w, -GLYPH],
        [w, -GLYPH],
    ];
    let idx = [0, 1, 2, 3, 5, 4, 4, 5, 6];
    tris(&verts, &idx, 0.0)
}

/// A cog / gear silhouette for the tools submenu.
#[must_use]
pub fn gear_mesh() -> MeshData {
    let teeth = 8;
    let r_tip = GLYPH;
    let r_valley = GLYPH * 0.68;
    let points = (0..teeth * 2)
        .map(|k| {
            let angle = k as f32 * PI / teeth as f32;
            let r = if k % 2 == 0 { r_tip } else { r_valley };
            [r * angle.cos(), r * angle.sin()]
        })
        .collect::<Vec<_>>();
    fan(&points, 0.0)
}

/// A left-pointing chevron for the back sector.
#[must_use]
pub fn chevron_mesh() -> MeshData {
    let w = GLYPH * 0.9;
    let h = GLYPH;
    let t = GLYPH * 0.5;
    let verts = [
        [t, h],
        [t - w, 0.0],
        [t, -h],
        [t + t, h],
        [t + t - w, 0.0],
        [t + t, -h],
    ];
    let idx = [0, 1, 3, 3, 1, 4, 1, 2, 4, 4, 2, 5];
    tris(&verts, &idx, 0.0)
}

/// A map-pin (circular head over a tapered point) for the nav table.
#[must_use]
pub fn pin_mesh() -> MeshData {
    let head_r = GLYPH * 0.62;
    let head_cy = GLYPH * 0.32;
    let segments = 10;

    let mut positions = vec![0.0, head_cy, 0.0];
    let mut normals = vec![0.0, 0.0, 1.0];
    for i in 0..=segments {
        let angle = i as f32 / segments as f32 * TAU;
        positions.extend_from_slice(&[
            head_r.mul_add(angle.cos(), 0.0),
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
    let tail_base_y = head_cy - head_r * 0.55;
    let base = (positions.len() / 3) as u32;
    positions.extend_from_slice(&[
        -tail_w,
        tail_base_y,
        0.0,
        tail_w,
        tail_base_y,
        0.0,
        0.0,
        -GLYPH,
        0.0,
    ]);
    normals.extend_from_slice(&[0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0]);
    indices.extend_from_slice(&[base, base + 1, base + 2]);

    (positions, normals, indices)
}

/// A small diamond for an individual tool.
#[must_use]
pub fn diamond_mesh() -> MeshData {
    let r = GLYPH * 0.85;
    let verts = [[0.0, r], [r, 0.0], [0.0, -r], [-r, 0.0]];
    let idx = [0, 3, 1, 1, 3, 2];
    tris(&verts, &idx, 0.0)
}

/// A checkmark; the miter join keeps the tip sharp with no overhang.
#[must_use]
pub fn check_mesh() -> MeshData {
    let t = GLYPH * 0.36;
    let half = 1.0_f32.atan2(0.9);
    let bisector = PI / 2.0;
    let crux = [-GLYPH * 0.26, -GLYPH * 0.48];
    let p0 = arm(crux, bisector + half, GLYPH * 0.72);
    let p2 = arm(crux, bisector - half, GLYPH * 1.45);

    let n0 = left_normal(p0, crux);
    let n1 = left_normal(crux, p2);
    let m = normalize([n0[0] + n1[0], n0[1] + n1[1]]);
    let ml = t * 0.5 / m[1].mul_add(n0[1], m[0] * n0[0]);
    let h0 = [n0[0] * t * 0.5, n0[1] * t * 0.5];
    let h1 = [n1[0] * t * 0.5, n1[1] * t * 0.5];
    let mm = [m[0] * ml, m[1] * ml];

    let verts = [
        [p0[0] + h0[0], p0[1] + h0[1]],
        [crux[0] + mm[0], crux[1] + mm[1]],
        [p2[0] + h1[0], p2[1] + h1[1]],
        [p2[0] - h1[0], p2[1] - h1[1]],
        [crux[0] - mm[0], crux[1] - mm[1]],
        [p0[0] - h0[0], p0[1] - h0[1]],
    ];
    let idx = [0, 4, 1, 0, 5, 4, 1, 3, 2, 1, 4, 3];
    tris(&verts, &idx, 0.0)
}

fn arm(from: [f32; 2], angle: f32, len: f32) -> [f32; 2] {
    [
        len.mul_add(angle.cos(), from[0]),
        len.mul_add(angle.sin(), from[1]),
    ]
}

fn left_normal(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    let d = normalize([b[0] - a[0], b[1] - a[1]]);
    [-d[1], d[0]]
}

fn normalize(v: [f32; 2]) -> [f32; 2] {
    let len = v[0].hypot(v[1]).max(1.0e-5);
    [v[0] / len, v[1] / len]
}
