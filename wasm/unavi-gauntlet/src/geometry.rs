use std::f32::consts::PI;

use crate::wired::scene::types::Prim;

pub const SECTOR_INNER_R: f32 = 0.05;
pub const RING_RADIUS: f32 = 0.16;
pub const SECTOR_GAP_WORLD: f32 = 0.012;
pub const SECTOR_SUBDIVISIONS: usize = 40;
pub const OUTLINE_WIDTH: f32 = 0.006;
pub const OUTLINE_Z: f32 = 0.001;
pub const ICON_R: f32 = (SECTOR_INNER_R + RING_RADIUS) * 0.5;

/// (positions, normals, indices) for a flat mesh in the wheel plane.
pub type MeshData = (Vec<f32>, Vec<f32>, Vec<u32>);

pub fn apply_mesh(prim: &Prim, mesh: &MeshData) {
    prim.set_mesh_stream("POSITION", Some(&mesh.0));
    prim.set_mesh_stream("NORMAL", Some(&mesh.1));
    prim.set_mesh_indices_u32(Some(&mesh.2));
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

fn annulus(i: usize, n: usize, r_inner: f32, r_outer: f32, z: f32) -> MeshData {
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
            let t = j as f32 / subs as f32;
            let a = t.mul_add(end - start, start);
            positions.extend_from_slice(&[r * a.cos(), r * a.sin(), z]);
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
/// Every glyph is padded to this vertex count so a pooled glyph prim never
/// changes its POSITION/NORMAL length, avoiding transient count mismatches when
/// its icon changes (mesh streams commit independently).
const GLYPH_VERTS: usize = 24;

fn pad(mut mesh: MeshData) -> MeshData {
    let verts = mesh.0.len() / 3;
    let (lx, ly, lz) = mesh.0.last_chunk::<3>().map_or((0.0, 0.0, 0.0), |c| {
        (c[0], c[1], c[2])
    });
    for _ in verts..GLYPH_VERTS {
        mesh.0.extend_from_slice(&[lx, ly, lz]);
        mesh.1.extend_from_slice(&[0.0, 0.0, 1.0]);
    }
    mesh
}

/// A house silhouette (roof triangle over a wall block).
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
    pad(tris(&verts, &idx, 0.0))
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
    pad(fan(&points, 0.0))
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
    pad(tris(&verts, &idx, 0.0))
}

/// A small diamond for an individual tool.
#[must_use]
pub fn diamond_mesh() -> MeshData {
    let r = GLYPH * 0.85;
    let verts = [[0.0, r], [r, 0.0], [0.0, -r], [-r, 0.0]];
    let idx = [0, 3, 1, 1, 3, 2];
    pad(tris(&verts, &idx, 0.0))
}

/// A mitered checkmark for the home-travel confirmation.
#[must_use]
pub fn check_mesh() -> MeshData {
    let t = GLYPH * 0.34;
    let p0 = [-GLYPH * 0.7, 0.0];
    let p1 = [-GLYPH * 0.15, -GLYPH * 0.62];
    let p2 = [GLYPH * 0.85, GLYPH * 0.72];

    let n0 = left_normal(p0, p1);
    let n1 = left_normal(p1, p2);
    let miter = normalize([n0[0] + n1[0], n0[1] + n1[1]]);
    let denom = (miter[0] * n0[0] + miter[1] * n0[1]).max(0.35);
    let ml = t * 0.5 / denom;
    let h0 = [n0[0] * t * 0.5, n0[1] * t * 0.5];
    let h1 = [n1[0] * t * 0.5, n1[1] * t * 0.5];
    let m = [miter[0] * ml, miter[1] * ml];

    let verts = [
        [p0[0] + h0[0], p0[1] + h0[1]], // 0 p0 left
        [p1[0] + m[0], p1[1] + m[1]],   // 1 miter left
        [p2[0] + h1[0], p2[1] + h1[1]], // 2 p2 left
        [p2[0] - h1[0], p2[1] - h1[1]], // 3 p2 right
        [p1[0] - m[0], p1[1] - m[1]],   // 4 miter right
        [p0[0] - h0[0], p0[1] - h0[1]], // 5 p0 right
    ];
    let idx = [0, 1, 4, 0, 4, 5, 1, 2, 3, 1, 3, 4];
    pad(tris(&verts, &idx, 0.0))
}

fn left_normal(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    let d = normalize([b[0] - a[0], b[1] - a[1]]);
    [-d[1], d[0]]
}

fn normalize(v: [f32; 2]) -> [f32; 2] {
    let len = v[0].hypot(v[1]).max(1.0e-5);
    [v[0] / len, v[1] / len]
}
