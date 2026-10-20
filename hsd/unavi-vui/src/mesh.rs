//! Mesh generators for the shapes a surface draws: bodies, pips, the
//! overflow and way-back markers, and the marks that say what kind of mote a
//! body is.

use std::f32::consts::{
    PI,
    TAU,
};

use crate::{
    glyph::Glyph,
    mote::{
        Arrange,
        MAX_PIPS,
    },
};

/// Corners of the polygon a lit dot is drawn as.
const DOT_SIDES: usize = 8;
/// A lit dot's radius, as a fraction of its cell.
const DOT_RADIUS: f32 = 0.42;

/// Positions, normals, texture coordinates and indices, in the shape the scene
/// API's mesh streams take.
///
/// The coordinates are what any effect written across a body reads — a sweep
/// around a ring, a dissolve up a mote — and every generator here supplies
/// them, since a shader graph reading `Uv` on a mesh without them silently
/// falls back to zero and the whole body shades alike.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MeshData {
    pub positions: Vec<f32>,
    pub normals:   Vec<f32>,
    pub uvs:       Vec<f32>,
    pub indices:   Vec<u32>,
}

/// A UV sphere.
#[must_use]
pub fn sphere(radius: f32, rings: usize, segments: usize) -> MeshData {
    let mut mesh = MeshData::default();
    push_sphere(&mut mesh, [0.0; 3], radius, rings, segments);
    mesh
}

/// `len` pips at positions `start..start + len` of `total`, in unit space, as
/// one mesh.
///
/// Always [`MAX_PIPS`] bodies; unused ones sit at zero radius so the vertex
/// count never changes.
#[must_use]
pub fn cluster(
    start: usize,
    len: usize,
    total: usize,
    arrange: Arrange,
    spread: f32,
    radius: f32,
) -> MeshData {
    const RINGS: usize = 6;
    const SEGMENTS: usize = 8;

    let end = start.saturating_add(len).min(total);
    let mut mesh = MeshData::default();
    for index in 0..MAX_PIPS {
        let shown = total > 0 && index >= start && index < end;
        let (centre, radius) = if shown {
            (pip_position(index, total, arrange, spread), radius)
        } else {
            ([0.0; 3], 0.0)
        };
        push_sphere(&mut mesh, centre, radius, RINGS, SEGMENTS);
    }
    mesh
}

/// Where pip `index` of `total` sits, in the shape the level it previews will
/// open into.
///
/// An empty level is asked where its first cell would be — a group that opens
/// as a grid holds nothing until a registry listing or a tool's library
/// answers, and it is drawn every frame until then. One is the smallest
/// arrangement that has a position at all.
fn pip_position(index: usize, total: usize, arrange: Arrange, spread: f32) -> [f32; 3] {
    let total = total.max(1);
    match arrange {
        Arrange::Orbit => {
            let angle = TAU * index as f32 / total as f32;
            let (sin, cos) = angle.sin_cos();
            [spread * sin, spread * cos, 0.0]
        }
        Arrange::Grid => {
            let columns = grid_columns(total);
            let rows = total.div_ceil(columns);
            // The longer side spans the spread, so a grid of any shape sits
            // inside the same body a ring would.
            let pitch = 2.0 * spread / columns.max(rows) as f32;
            let (column, row) = (index % columns, index / columns);
            [
                ((columns - 1) as f32).mul_add(-0.5, column as f32) * pitch,
                ((rows - 1) as f32).mul_add(0.5, -(row as f32)) * pitch,
                0.0,
            ]
        }
    }
}

/// Squarest grid that holds `total`, widthways first.
fn grid_columns(total: usize) -> usize {
    (total as f32).sqrt().ceil() as usize
}

/// The marker saying a container holds more than its pips can show.
#[must_use]
pub fn overflow_marker(radius: f32) -> MeshData {
    sphere(radius, 6, 8)
}

/// Where that marker sits, in the same unit space as the pips.
///
/// A ring leaves its middle empty, so the marker goes there. A grid has no
/// spare middle, so it takes the cell after the last pip — which is why a
/// grid draws one pip fewer once it overflows.
#[must_use]
pub fn overflow_at(arrange: Arrange, total: usize, spread: f32) -> [f32; 3] {
    match arrange {
        Arrange::Orbit => [0.0; 3],
        Arrange::Grid => pip_position(total, total, arrange, spread),
    }
}

/// A toggle's mark: a short horizontal bar just below the face centre, in the
/// same unit space as the body.
#[must_use]
pub fn toggle_bar() -> MeshData {
    const HALF_WIDTH: f32 = 0.36;
    const HALF_HEIGHT: f32 = 0.07;
    const CENTRE: f32 = -0.3;
    const HALF_DEPTH: f32 = 0.06;

    let (low, high) = (CENTRE - HALF_HEIGHT, CENTRE + HALF_HEIGHT);
    let outline = [
        [-HALF_WIDTH, low],
        [HALF_WIDTH, low],
        [HALF_WIDTH, high],
        [-HALF_WIDTH, high],
    ];
    prism(&outline, &[[0, 1, 2], [0, 2, 3]], HALF_DEPTH)
}

/// A unit box centred on the origin: a tether's bar, scaled to its width and
/// its length along Z.
#[must_use]
pub fn bar() -> MeshData {
    let outline = [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]];
    prism(&outline, &[[0, 1, 2], [0, 2, 3]], 0.5)
}

/// Where a cast's resting ring stands, as fractions of its body's radius: just
/// outside the shell, and inside the ring a cast site draws once it fills.
pub const CAST_RING_INNER: f32 = 1.055;
pub const CAST_RING_OUTER: f32 = 1.085;

/// A cast's mark: a thin ring hugging the body's silhouette, in unit space.
#[must_use]
pub fn cast_ring() -> MeshData {
    annulus(CAST_RING_INNER, CAST_RING_OUTER, 48)
}

/// Extrudes a counter-clockwise `outline` along Z into a solid, `triangles`
/// indexing the outline to fill its face.
fn prism(outline: &[[f32; 2]], triangles: &[[usize; 3]], half_depth: f32) -> MeshData {
    let (min, max) = bounds(outline);
    let span = [
        (max[0] - min[0]).max(f32::EPSILON),
        (max[1] - min[1]).max(f32::EPSILON),
    ];
    let uv = |point: [f32; 2]| [(point[0] - min[0]) / span[0], (point[1] - min[1]) / span[1]];

    let mut mesh = MeshData::default();
    for facing in [1.0_f32, -1.0] {
        let base = (mesh.positions.len() / 3) as u32;
        for point in outline {
            mesh.positions
                .extend_from_slice(&[point[0], point[1], facing * half_depth]);
            mesh.normals.extend_from_slice(&[0.0, 0.0, facing]);
            mesh.uvs.extend_from_slice(&uv(*point));
        }
        for face in triangles {
            let [a, b, c] = face.map(|index| base + index as u32);
            let wound = if facing > 0.0 { [a, b, c] } else { [a, c, b] };
            mesh.indices.extend_from_slice(&wound);
        }
    }

    for (index, from) in outline.iter().enumerate() {
        let to = outline[(index + 1) % outline.len()];
        let edge = [to[0] - from[0], to[1] - from[1]];
        let length = edge[0].hypot(edge[1]).max(f32::EPSILON);
        let normal = [edge[1] / length, -edge[0] / length, 0.0];

        let base = (mesh.positions.len() / 3) as u32;
        for (point, depth) in [
            (from, half_depth),
            (&to, half_depth),
            (&to, -half_depth),
            (from, -half_depth),
        ] {
            mesh.positions
                .extend_from_slice(&[point[0], point[1], depth]);
            mesh.normals.extend_from_slice(&normal);
            mesh.uvs.extend_from_slice(&uv(*point));
        }
        mesh.indices
            .extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
    }

    mesh
}

fn bounds(outline: &[[f32; 2]]) -> ([f32; 2], [f32; 2]) {
    outline
        .iter()
        .fold(([f32::MAX; 2], [f32::MIN; 2]), |(min, max), point| {
            (
                [min[0].min(point[0]), min[1].min(point[1])],
                [max[0].max(point[0]), max[1].max(point[1])],
            )
        })
}

/// A unit quad facing +Z, its top edge on the origin, descending.
#[must_use]
pub fn panel() -> MeshData {
    MeshData {
        positions: vec![
            -0.5, 0.0, 0.0, //
            0.5, 0.0, 0.0, //
            -0.5, -1.0, 0.0, //
            0.5, -1.0, 0.0,
        ],
        normals:   [0.0, 0.0, 1.0].repeat(4),
        uvs:       vec![
            0.0, 1.0, //
            1.0, 1.0, //
            0.0, 0.0, //
            1.0, 0.0,
        ],
        indices:   vec![0, 2, 1, 1, 2, 3],
    }
}

/// A unit square facing +Z, centred on the origin: the surface a glyph's dots
/// are shaded across.
///
/// u runs left to right and v top to bottom, the order a glyph's cells are
/// numbered in, so the shader can turn a coordinate straight into a row and
/// column.
#[must_use]
pub fn face() -> MeshData {
    MeshData {
        positions: vec![
            -0.5, 0.5, 0.0, //
            0.5, 0.5, 0.0, //
            -0.5, -0.5, 0.0, //
            0.5, -0.5, 0.0,
        ],
        normals:   [0.0, 0.0, 1.0].repeat(4),
        uvs:       vec![
            0.0, 0.0, //
            1.0, 0.0, //
            0.0, 1.0, //
            1.0, 1.0,
        ],
        indices:   vec![0, 2, 1, 1, 2, 3],
    }
}

/// A glyph's face as a cap of the unit sphere, so its dots sit on the glass
/// they are drawn for rather than on a flat pane standing off it.
///
/// `fill` is the half-width the grid's UV spans, so the grid keeps the size it
/// had on a flat face; the cap reaches `reach` in-plane, past the grid's clip
/// circle, so its own edge is never seen. u runs left to right and v top to
/// bottom, the same as [`face`].
#[must_use]
pub fn dome(fill: f32, reach: f32) -> MeshData {
    const RINGS: usize = 6;
    const SEGMENTS: usize = 32;

    let mut mesh = MeshData::default();
    // The pole, then rings out to the cap's edge.
    mesh.positions.extend_from_slice(&[0.0, 0.0, 1.0]);
    mesh.normals.extend_from_slice(&[0.0, 0.0, 1.0]);
    mesh.uvs.extend_from_slice(&[0.5, 0.5]);
    for ring in 1..=RINGS {
        let r = reach * ring as f32 / RINGS as f32;
        let z = r.mul_add(-r, 1.0).sqrt();
        for segment in 0..=SEGMENTS {
            let theta = TAU * segment as f32 / SEGMENTS as f32;
            let (sin, cos) = theta.sin_cos();
            let (x, y) = (r * cos, r * sin);
            mesh.positions.extend_from_slice(&[x, y, z]);
            // A unit sphere's normal is its own position.
            mesh.normals.extend_from_slice(&[x, y, z]);
            mesh.uvs
                .extend_from_slice(&[x / (2.0 * fill) + 0.5, 0.5 - y / (2.0 * fill)]);
        }
    }

    let stride = (SEGMENTS + 1) as u32;
    for segment in 0..SEGMENTS as u32 {
        mesh.indices
            .extend_from_slice(&[0, 1 + segment, 1 + segment + 1]);
    }
    for ring in 0..RINGS as u32 - 1 {
        for segment in 0..SEGMENTS as u32 {
            let a = 1 + ring * stride + segment;
            let b = a + stride;
            mesh.indices
                .extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    mesh
}

fn push_sphere(mesh: &mut MeshData, centre: [f32; 3], radius: f32, rings: usize, segments: usize) {
    let rings = rings.max(2);
    let segments = segments.max(3);
    let base = (mesh.positions.len() / 3) as u32;

    for ring in 0..=rings {
        let ring_t = ring as f32 / rings as f32;
        let phi = PI * ring_t;
        let (sin_phi, cos_phi) = phi.sin_cos();
        for segment in 0..=segments {
            let segment_t = segment as f32 / segments as f32;
            let theta = TAU * segment_t;
            let (sin_theta, cos_theta) = theta.sin_cos();
            let normal = [sin_phi * cos_theta, cos_phi, sin_phi * sin_theta];
            mesh.normals.extend_from_slice(&normal);
            mesh.positions.extend_from_slice(&[
                normal[0].mul_add(radius, centre[0]),
                normal[1].mul_add(radius, centre[1]),
                normal[2].mul_add(radius, centre[2]),
            ]);
            // v runs 0 at the bottom, so anything written up a body — a
            // dissolve, a fill — travels the way a thing fills up.
            mesh.uvs.extend_from_slice(&[segment_t, 1.0 - ring_t]);
        }
    }

    let stride = (segments + 1) as u32;
    for ring in 0..rings as u32 {
        for segment in 0..segments as u32 {
            let a = base + ring * stride + segment;
            let b = a + stride;
            mesh.indices
                .extend_from_slice(&[a, b, a + 1, a + 1, b, b + 1]);
        }
    }
}

/// A flat annulus in the XY plane, for orbit guides and cast circles.
///
/// u is the angle and v crosses the band, so a fill sweeps round on u and an
/// edge softens on v — the coordinates a ring is actually shaded in, which no
/// amount of world position would give.
#[must_use]
pub fn annulus(inner: f32, outer: f32, segments: usize) -> MeshData {
    let segments = segments.max(3);
    let mut mesh = MeshData {
        positions: Vec::with_capacity((segments + 1) * 6),
        normals:   Vec::with_capacity((segments + 1) * 6),
        uvs:       Vec::with_capacity((segments + 1) * 4),
        indices:   Vec::with_capacity(segments * 6),
    };

    for segment in 0..=segments {
        let angle = segment as f32 / segments as f32;
        let theta = TAU * angle;
        let (sin, cos) = theta.sin_cos();
        for (edge, radius) in [inner, outer].into_iter().enumerate() {
            mesh.positions
                .extend_from_slice(&[radius * cos, radius * sin, 0.0]);
            mesh.normals.extend_from_slice(&[0.0, 0.0, 1.0]);
            mesh.uvs.extend_from_slice(&[angle, edge as f32]);
        }
    }

    for segment in 0..segments as u32 {
        let a = segment * 2;
        mesh.indices
            .extend_from_slice(&[a, a + 1, a + 2, a + 2, a + 1, a + 3]);
    }

    mesh
}

/// The lit dots of `glyph` on a unit square facing +Z and centred on the
/// origin, row 0 at the top. Unlit dots have no geometry.
#[must_use]
pub fn dots(glyph: &Glyph) -> MeshData {
    let size = glyph.size();
    let cell = 1.0 / size as f32;
    let radius = cell * DOT_RADIUS;
    let mut mesh = MeshData::default();
    for row in 0..size {
        for column in 0..size {
            if !glyph.lit(column, row) {
                continue;
            }
            let x = (column as f32 + 0.5).mul_add(cell, -0.5);
            let y = (row as f32 + 0.5).mul_add(-cell, 0.5);
            let first = (mesh.positions.len() / 3) as u32;
            mesh.positions.extend_from_slice(&[x, y, 0.0]);
            mesh.uvs.extend_from_slice(&[x + 0.5, 0.5 - y]);
            for side in 0..DOT_SIDES {
                let (sin, cos) = (TAU * side as f32 / DOT_SIDES as f32).sin_cos();
                let (px, py) = (radius.mul_add(cos, x), radius.mul_add(sin, y));
                mesh.positions.extend_from_slice(&[px, py, 0.0]);
                mesh.uvs.extend_from_slice(&[px + 0.5, 0.5 - py]);
                let this = first + 1 + side as u32;
                let next = first + 1 + ((side + 1) % DOT_SIDES) as u32;
                mesh.indices.extend_from_slice(&[first, this, next]);
            }
            mesh.normals.extend([0.0, 0.0, 1.0].repeat(DOT_SIDES + 1));
        }
    }
    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_lit_dots_have_geometry_and_they_stay_inside_the_square() {
        let glyph = Glyph::new(3, &[0b0001_0001, 0b0000_0001]).expect("a three grid");
        let mesh = dots(&glyph);
        assert_eq!(vertex_count(&mesh), 3 * (DOT_SIDES + 1), "cells 0, 4 and 8");
        assert_eq!(mesh.indices.len(), 3 * DOT_SIDES * 3);
        assert_eq!(mesh.normals.len(), mesh.positions.len());
        assert_eq!(mesh.uvs.len() / 2, vertex_count(&mesh));
        assert!(mesh.positions.iter().all(|v| v.abs() <= 0.5));
    }

    const SPREAD: f32 = 0.5;
    const RADIUS: f32 = 0.2;

    fn ring(start: usize, len: usize, total: usize) -> MeshData {
        cluster(start, len, total, Arrange::Orbit, SPREAD, RADIUS)
    }

    fn vertex_count(mesh: &MeshData) -> usize {
        mesh.positions.len() / 3
    }

    #[test]
    fn an_empty_grid_still_answers_where_its_first_cell_is() {
        // A group that opens as a grid is empty until whatever fills it
        // answers — a registry listing, a tool's library — and it is drawn
        // every frame in the meantime.
        for arrange in [Arrange::Grid, Arrange::Orbit] {
            assert!(
                overflow_at(arrange, 0, SPREAD)
                    .iter()
                    .all(|axis| axis.is_finite()),
                "an empty level must answer with a place, not a panic or a NaN"
            );
        }
    }

    #[test]
    fn an_empty_level_draws_no_pips() {
        let mesh = cluster(0, 0, 0, Arrange::Grid, SPREAD, RADIUS);
        assert_well_formed(&mesh);
        assert_eq!(
            mesh.positions.iter().filter(|p| **p != 0.0).count(),
            0,
            "every body collapses to the origin at zero radius"
        );
    }

    #[test]
    fn a_panel_hangs_from_its_top_edge() {
        let panel = panel();
        assert_well_formed(&panel);
        let ys = panel.positions.chunks(3).map(|p| p[1]).collect::<Vec<_>>();
        assert!(
            ys.iter().all(|y| *y <= 0.0),
            "a placard's layout descends from its top, so the mesh must too"
        );
        assert!(ys.iter().any(|y| (*y + 1.0).abs() < 1.0e-6));
    }

    fn assert_well_formed(mesh: &MeshData) {
        assert_eq!(mesh.positions.len(), mesh.normals.len());
        assert_eq!(mesh.positions.len() % 3, 0);
        assert_eq!(mesh.indices.len() % 3, 0);
        let vertices = vertex_count(mesh);
        // A stream whose vertex count disagrees with POSITION is refused
        // outright, so a body short of coordinates is not drawn at all.
        assert_eq!(
            mesh.uvs.len(),
            vertices * 2,
            "every vertex carries a texture coordinate"
        );
        for &index in &mesh.indices {
            assert!(index < vertices as u32, "index {index} out of range");
        }
        for value in mesh.positions.iter().chain(&mesh.normals) {
            assert!(value.is_finite());
        }
        for value in &mesh.uvs {
            assert!(
                value.is_finite() && (0.0..=1.0).contains(value),
                "a coordinate outside the unit square wraps or clamps rather \
                 than shading what it was written for: {value}"
            );
        }
    }

    #[test]
    fn a_panel_is_well_formed() {
        assert_well_formed(&panel());
    }

    #[test]
    fn a_face_is_a_centred_square_numbered_from_its_top_left() {
        let face = face();
        assert_well_formed(&face);
        assert_unit_normals(&face);
        // The top left vertex is the origin of the glyph's rows and columns.
        assert_eq!(face.positions[..3], [-0.5, 0.5, 0.0]);
        assert_eq!(face.uvs[..2], [0.0, 0.0]);
        assert_eq!(face.uvs[6..], [1.0, 1.0]);
    }

    #[test]
    fn an_overflow_marker_is_well_formed() {
        assert_well_formed(&overflow_marker(0.11));
    }

    /// The convention anything written up a body depends on: a mote fills from
    /// the bottom, so its lowest vertex is where a dissolve or a fill starts.
    #[test]
    fn a_spheres_coordinates_run_up_from_its_bottom() {
        let mesh = sphere(1.0, 8, 12);
        let (lowest, highest) = mesh
            .positions
            .as_chunks::<3>()
            .0
            .iter()
            .zip(mesh.uvs.as_chunks::<2>().0)
            .fold(
                ((f32::MAX, 0.0_f32), (f32::MIN, 0.0_f32)),
                |(low, high), (point, uv)| {
                    let low = if point[1] < low.0 {
                        (point[1], uv[1])
                    } else {
                        low
                    };
                    let high = if point[1] > high.0 {
                        (point[1], uv[1])
                    } else {
                        high
                    };
                    (low, high)
                },
            );
        assert!(lowest.1 < 0.01, "the bottom is v = 0, got {}", lowest.1);
        assert!(highest.1 > 0.99, "the top is v = 1, got {}", highest.1);
    }

    /// A sweep reads u and an edge softens on v, so the two must not be the
    /// same thing measured twice.
    #[test]
    fn an_annulus_carries_the_angle_on_u_and_the_band_on_v() {
        let (inner, outer) = (0.04_f32, 0.06_f32);
        let mesh = annulus(inner, outer, 16);
        for (point, uv) in mesh
            .positions
            .as_chunks::<3>()
            .0
            .iter()
            .zip(mesh.uvs.as_chunks::<2>().0)
        {
            let radius = point[0].hypot(point[1]);
            let expected = f32::from(radius > inner.midpoint(outer));
            assert!(
                (uv[1] - expected).abs() < 1.0e-6,
                "v names which edge a vertex is on"
            );
            let angle = point[1].atan2(point[0]).rem_euclid(TAU) / TAU;
            let apart = (uv[0] - angle).abs();
            assert!(
                apart < 1.0e-4 || (apart - 1.0).abs() < 1.0e-4,
                "u is the angle round the ring; {} vs {angle}",
                uv[0]
            );
        }
    }

    #[test]
    fn a_sphere_is_well_formed() {
        assert_well_formed(&sphere(0.05, 12, 18));
    }

    #[test]
    fn sphere_vertices_sit_on_the_radius() {
        let radius = 0.07;
        let mesh = sphere(radius, 8, 12);
        for point in mesh.positions.as_chunks::<3>().0 {
            let length = point[0].hypot(point[1]).hypot(point[2]);
            assert!((length - radius).abs() < 1.0e-4, "got {length}");
        }
    }

    #[test]
    fn sphere_normals_are_unit_length() {
        let mesh = sphere(0.05, 8, 12);
        for normal in mesh.normals.as_chunks::<3>().0 {
            let length = normal[0].hypot(normal[1]).hypot(normal[2]);
            assert!((length - 1.0).abs() < 1.0e-4, "got {length}");
        }
    }

    #[test]
    fn degenerate_sphere_parameters_are_clamped_not_panicked() {
        let mesh = sphere(0.05, 0, 0);
        assert_well_formed(&mesh);
        assert_ne!(mesh.indices, [] as [u32; 0]);
    }

    /// A glyph's cap wraps the unit sphere, and its UV is the planar grid the
    /// flat face used, so a glyph keeps its size and its cell layout.
    #[test]
    fn a_dome_wraps_the_unit_sphere_with_a_planar_grid() {
        let fill = 0.62;
        let mesh = dome(fill, fill * 1.05);
        assert_eq!(mesh.positions.len(), mesh.normals.len());
        assert_eq!(mesh.positions.len() % 3, 0);
        assert_eq!(mesh.indices.len() % 3, 0);
        let vertices = vertex_count(&mesh);
        assert_eq!(mesh.uvs.len(), vertices * 2);
        for &index in &mesh.indices {
            assert!(index < vertices as u32, "index {index} out of range");
        }
        for (point, uv) in mesh
            .positions
            .as_chunks::<3>()
            .0
            .iter()
            .zip(mesh.uvs.as_chunks::<2>().0)
        {
            let length = point[0].hypot(point[1]).hypot(point[2]);
            assert!((length - 1.0).abs() < 1.0e-4, "off the sphere: {length}");
            let expected = (point[0] / (2.0 * fill) + 0.5, 0.5 - point[1] / (2.0 * fill));
            assert!((uv[0] - expected.0).abs() < 1.0e-5, "u {}", uv[0]);
            assert!((uv[1] - expected.1).abs() < 1.0e-5, "v {}", uv[1]);
        }
    }

    /// Bodies drawn away from the origin, once hidden pips collapse to zero
    /// radius there.
    fn visible_bodies(mesh: &MeshData) -> usize {
        mesh.positions
            .as_chunks::<3>()
            .0
            .iter()
            .filter(|point| point[0].hypot(point[1]).hypot(point[2]) > 1.0e-6)
            .count()
            / 63
    }

    #[test]
    fn a_cluster_shows_one_body_per_pip() {
        assert_eq!(visible_bodies(&ring(0, 1, 3)), 1);
        assert_eq!(visible_bodies(&ring(0, 3, 3)), 3);
        assert_well_formed(&ring(0, 3, 3));
    }

    #[test]
    fn every_cluster_has_the_same_vertex_count() {
        // Vertex streams are separate entries, so a rebuild that changes the
        // count is briefly inconsistent and the host rejects the mesh.
        let baseline = vertex_count(&ring(0, 0, 0));
        for (start, len, total) in [(0, 1, 1), (0, 7, 7), (2, 3, 5), (3, 1, 4)] {
            assert_eq!(
                vertex_count(&ring(start, len, total)),
                baseline,
                "{start}..{len} of {total} changed the vertex count"
            );
        }
    }

    #[test]
    fn cluster_indices_address_their_own_body() {
        // A merged mesh whose later spheres still index the first one draws
        // garbage; this is the whole risk of merging.
        let mesh = ring(0, 4, 4);
        assert_well_formed(&mesh);
        let highest = mesh.indices.iter().copied().max().expect("indices");
        assert_eq!(
            highest as usize,
            vertex_count(&mesh) - 1,
            "the last body's vertices are reached"
        );
    }

    #[test]
    fn two_runs_tile_the_ring_without_overlapping() {
        // How a branch draws container children see-through and leaf children
        // solid: two meshes, same ring.
        let leading = ring(0, 2, 5);
        let trailing = ring(2, 3, 5);
        assert_eq!(visible_bodies(&leading), 2);
        assert_eq!(visible_bodies(&trailing), 3);
        assert_eq!(
            visible_bodies(&leading) + visible_bodies(&trailing),
            visible_bodies(&ring(0, 5, 5))
        );
        assert_well_formed(&trailing);
    }

    #[test]
    fn a_run_past_the_ring_is_clamped_rather_than_wrapping() {
        assert_eq!(visible_bodies(&ring(3, 9, 4)), 1);
    }

    #[test]
    fn an_empty_cluster_draws_nothing() {
        for mesh in [ring(0, 0, 0), ring(0, 3, 0)] {
            assert_eq!(visible_bodies(&mesh), 0);
            assert_well_formed(&mesh);
        }
    }

    /// Pip centres, in the order they were laid out.
    fn centres(mesh: &MeshData, count: usize) -> Vec<[f32; 3]> {
        // Every body is 63 vertices; its first is enough to place it, since a
        // sphere's vertices straddle its centre in x and z.
        mesh.positions
            .as_chunks::<{ 63 * 3 }>()
            .0
            .iter()
            .take(count)
            .map(|body| {
                let (mut low, mut high) = ([f32::MAX; 3], [f32::MIN; 3]);
                for point in body.as_chunks::<3>().0 {
                    for axis in 0..3 {
                        low[axis] = low[axis].min(point[axis]);
                        high[axis] = high[axis].max(point[axis]);
                    }
                }
                [
                    f32::midpoint(low[0], high[0]),
                    f32::midpoint(low[1], high[1]),
                    f32::midpoint(low[2], high[2]),
                ]
            })
            .collect()
    }

    #[test]
    fn a_grid_cluster_lays_its_pips_out_in_rows() {
        let mesh = cluster(0, 4, 4, Arrange::Grid, SPREAD, RADIUS);
        assert_well_formed(&mesh);

        let centres = centres(&mesh, 4);
        assert!(
            centres[0][1] > centres[2][1],
            "the first row sits above the second"
        );
        assert!(
            centres[0][0] < centres[1][0],
            "and reads left to right within a row"
        );
        assert!(
            centres
                .iter()
                .all(|c| c[0].abs() <= SPREAD + 1.0e-5 && c[1].abs() <= SPREAD + 1.0e-5),
            "a grid stays inside the body a ring would fill"
        );
    }

    #[test]
    fn a_ring_and_a_grid_of_the_same_contents_place_them_differently() {
        let ringed = centres(&ring(0, 5, 5), 5);
        let gridded = centres(&cluster(0, 5, 5, Arrange::Grid, SPREAD, RADIUS), 5);
        assert_ne!(
            ringed, gridded,
            "the preview is the promise of how the level opens"
        );
    }

    fn radius_of(point: &[f32]) -> f32 {
        point[0].hypot(point[1]).hypot(point[2])
    }

    fn assert_unit_normals(mesh: &MeshData) {
        for normal in mesh.normals.as_chunks::<3>().0 {
            assert!(
                (radius_of(normal) - 1.0).abs() < 1.0e-4,
                "a normal is a direction: {normal:?}"
            );
        }
    }

    #[test]
    fn a_toggle_bar_is_a_short_bar_below_the_centre_inside_the_body() {
        let mesh = toggle_bar();
        assert_well_formed(&mesh);
        assert_unit_normals(&mesh);

        let points = mesh.positions.as_chunks::<3>().0;
        assert!(
            points.iter().all(|point| point[1] < 0.0),
            "below the centre"
        );
        assert!(
            points.iter().all(|point| radius_of(point) < 0.8),
            "inside the body"
        );
        let width = points.iter().map(|p| p[0].abs()).fold(0.0_f32, f32::max);
        let height = points.iter().map(|p| p[1]).fold(f32::MIN, f32::max)
            - points.iter().map(|p| p[1]).fold(f32::MAX, f32::min);
        assert!(width * 2.0 > height * 3.0, "a bar, not a block");
    }

    #[test]
    fn a_cast_ring_hugs_the_outside_of_the_body() {
        let mesh = cast_ring();
        assert_well_formed(&mesh);
        assert_unit_normals(&mesh);
        for point in mesh.positions.as_chunks::<3>().0 {
            let radius = radius_of(point);
            assert!(
                (CAST_RING_INNER - 1.0e-4..=CAST_RING_OUTER + 1.0e-4).contains(&radius),
                "{radius}: outside the shell and inside the cast site's ring"
            );
        }
    }

    #[test]
    fn an_annulus_is_well_formed() {
        assert_well_formed(&annulus(0.04, 0.05, 24));
    }

    #[test]
    fn annulus_vertices_sit_between_its_radii() {
        let (inner, outer) = (0.04_f32, 0.06_f32);
        let mesh = annulus(inner, outer, 16);
        for point in mesh.positions.as_chunks::<3>().0 {
            let radius = point[0].hypot(point[1]);
            assert!(radius >= inner - 1.0e-4 && radius <= outer + 1.0e-4);
            assert!((point[2]).abs() < 1.0e-6, "the annulus is flat");
        }
    }
}
