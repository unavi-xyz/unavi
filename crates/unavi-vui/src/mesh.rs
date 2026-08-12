use std::f32::consts::{
    PI,
    TAU,
};

use crate::mote::{
    Arrange,
    MAX_PIPS,
};

/// Positions, normals and indices, in the shape the scene API's mesh streams
/// take.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MeshData {
    pub positions: Vec<f32>,
    pub normals:   Vec<f32>,
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
fn pip_position(index: usize, total: usize, arrange: Arrange, spread: f32) -> [f32; 3] {
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
        indices:   vec![0, 2, 1, 1, 2, 3],
    }
}

fn push_sphere(mesh: &mut MeshData, centre: [f32; 3], radius: f32, rings: usize, segments: usize) {
    let rings = rings.max(2);
    let segments = segments.max(3);
    let base = (mesh.positions.len() / 3) as u32;

    for ring in 0..=rings {
        let phi = PI * ring as f32 / rings as f32;
        let (sin_phi, cos_phi) = phi.sin_cos();
        for segment in 0..=segments {
            let theta = TAU * segment as f32 / segments as f32;
            let (sin_theta, cos_theta) = theta.sin_cos();
            let normal = [sin_phi * cos_theta, cos_phi, sin_phi * sin_theta];
            mesh.normals.extend_from_slice(&normal);
            mesh.positions.extend_from_slice(&[
                normal[0].mul_add(radius, centre[0]),
                normal[1].mul_add(radius, centre[1]),
                normal[2].mul_add(radius, centre[2]),
            ]);
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
#[must_use]
pub fn annulus(inner: f32, outer: f32, segments: usize) -> MeshData {
    let segments = segments.max(3);
    let mut mesh = MeshData {
        positions: Vec::with_capacity((segments + 1) * 6),
        normals:   Vec::with_capacity((segments + 1) * 6),
        indices:   Vec::with_capacity(segments * 6),
    };

    for segment in 0..=segments {
        let theta = TAU * segment as f32 / segments as f32;
        let (sin, cos) = theta.sin_cos();
        for radius in [inner, outer] {
            mesh.positions
                .extend_from_slice(&[radius * cos, radius * sin, 0.0]);
            mesh.normals.extend_from_slice(&[0.0, 0.0, 1.0]);
        }
    }

    for segment in 0..segments as u32 {
        let a = segment * 2;
        mesh.indices
            .extend_from_slice(&[a, a + 1, a + 2, a + 2, a + 1, a + 3]);
    }

    mesh
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPREAD: f32 = 0.5;
    const RADIUS: f32 = 0.2;

    fn ring(start: usize, len: usize, total: usize) -> MeshData {
        cluster(start, len, total, Arrange::Orbit, SPREAD, RADIUS)
    }

    fn vertex_count(mesh: &MeshData) -> usize {
        mesh.positions.len() / 3
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
        let vertices = vertex_count(mesh) as u32;
        for &index in &mesh.indices {
            assert!(index < vertices, "index {index} out of range");
        }
        for value in mesh.positions.iter().chain(&mesh.normals) {
            assert!(value.is_finite());
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
        for point in mesh.positions.chunks_exact(3) {
            let length = point[0].hypot(point[1]).hypot(point[2]);
            assert!((length - radius).abs() < 1.0e-4, "got {length}");
        }
    }

    #[test]
    fn sphere_normals_are_unit_length() {
        let mesh = sphere(0.05, 8, 12);
        for normal in mesh.normals.chunks_exact(3) {
            let length = normal[0].hypot(normal[1]).hypot(normal[2]);
            assert!((length - 1.0).abs() < 1.0e-4, "got {length}");
        }
    }

    #[test]
    fn degenerate_sphere_parameters_are_clamped_not_panicked() {
        let mesh = sphere(0.05, 0, 0);
        assert_well_formed(&mesh);
        assert!(!mesh.indices.is_empty());
    }

    /// Bodies drawn away from the origin, once hidden pips collapse to zero
    /// radius there.
    fn visible_bodies(mesh: &MeshData) -> usize {
        mesh.positions
            .chunks_exact(3)
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
            .chunks_exact(63 * 3)
            .take(count)
            .map(|body| {
                let (mut low, mut high) = ([f32::MAX; 3], [f32::MIN; 3]);
                for point in body.chunks_exact(3) {
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

    #[test]
    fn an_annulus_is_well_formed() {
        assert_well_formed(&annulus(0.04, 0.05, 24));
    }

    #[test]
    fn annulus_vertices_sit_between_its_radii() {
        let (inner, outer) = (0.04_f32, 0.06_f32);
        let mesh = annulus(inner, outer, 16);
        for point in mesh.positions.chunks_exact(3) {
            let radius = point[0].hypot(point[1]);
            assert!(radius >= inner - 1.0e-4 && radius <= outer + 1.0e-4);
            assert!((point[2]).abs() < 1.0e-6, "the annulus is flat");
        }
    }
}
