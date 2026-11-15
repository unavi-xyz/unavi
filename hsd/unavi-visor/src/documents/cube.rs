//! The unit cube every bracket bar is scaled from.

use crate::wired::scene::properties::{
    Property,
    Topology,
    VertexAttribute,
    VertexStream,
};

/// Face normals with the two edge directions spanning each face.
const FACES: [([f32; 3], [f32; 3], [f32; 3]); 6] = [
    ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]),
    ([-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]),
    ([0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
    ([0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
    ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
    ([0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]),
];

/// A cube of side one centred on the origin: 24 vertices, 36 indices.
pub struct Cube {
    pub positions: Vec<f32>,
    pub normals:   Vec<f32>,
    pub uvs:       Vec<f32>,
    pub indices:   Vec<u32>,
}

impl Cube {
    #[must_use]
    pub fn new() -> Self {
        let mut cube = Self {
            positions: Vec::new(),
            normals:   Vec::new(),
            uvs:       Vec::new(),
            indices:   Vec::new(),
        };
        for (face, (normal, u, v)) in FACES.iter().enumerate() {
            let base = (face * 4) as u32;
            for (du, dv) in [(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5_f32)] {
                for axis in 0..3 {
                    cube.positions
                        .push(normal[axis].mul_add(0.5, u[axis].mul_add(du, v[axis] * dv)));
                }
                cube.normals.extend_from_slice(normal);
                cube.uvs.extend_from_slice(&[du + 0.5, dv + 0.5]);
            }
            cube.indices
                .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
        cube
    }

    /// The properties that give a prim this mesh.
    #[must_use]
    pub fn properties(&self) -> [Property; 5] {
        let stream = |attribute, values: &[f32]| {
            Property::MeshVertices(VertexStream {
                attribute,
                values: values.to_vec(),
            })
        };
        [
            Property::MeshTopology(Topology::TriangleList),
            stream(VertexAttribute::Position, &self.positions),
            stream(VertexAttribute::Normal, &self.normals),
            stream(VertexAttribute::Uv0, &self.uvs),
            Property::MeshIndices(self.indices.clone()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cube_has_24_vertices_inside_a_unit_box_and_valid_indices() {
        let cube = Cube::new();
        assert_eq!(cube.positions.len(), 72);
        assert_eq!(cube.normals.len(), 72);
        assert_eq!(cube.uvs.len(), 48);
        assert_eq!(cube.indices.len(), 36);
        assert!(
            cube.positions
                .iter()
                .all(|p| (p.abs() - 0.5).abs() < 1.0e-6)
        );
        assert!(cube.indices.iter().all(|&index| index < 24));
    }
}
