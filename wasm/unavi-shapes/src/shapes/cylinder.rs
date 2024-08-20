use std::cell::Cell;

use crate::bindings::{
    exports::unavi::shapes::api::GuestCylinder,
    wired::{
        physics::types::{Collider, Shape, ShapeCylinder},
        scene::{mesh::Mesh, node::Node},
    },
};

pub struct Cylinder {
    cap: Cell<bool>,
    height: Cell<f32>,
    radius: Cell<f32>,
    resolution: Cell<u8>,
    segments: Cell<u8>,
}

impl GuestCylinder for Cylinder {
    fn new(radius: f32, height: f32) -> Self {
        Self {
            cap: Cell::new(true),
            height: Cell::new(height),
            radius: Cell::new(radius),
            resolution: Cell::new(32),
            segments: Cell::new(1),
        }
    }

    fn cap(&self) -> bool {
        self.cap.get()
    }
    fn set_cap(&self, value: bool) {
        self.cap.set(value);
    }

    fn height(&self) -> f32 {
        self.height.get()
    }
    fn set_height(&self, value: f32) {
        self.height.set(value);
    }

    fn radius(&self) -> f32 {
        self.radius.get()
    }
    fn set_radius(&self, value: f32) {
        self.radius.set(value);
    }

    fn resolution(&self) -> u8 {
        self.resolution.get()
    }
    fn set_resolution(&self, value: u8) {
        self.resolution.set(value);
    }

    fn segments(&self) -> u8 {
        self.segments.get()
    }
    fn set_segments(&self, value: u8) {
        self.segments.set(value);
    }

    fn to_mesh(&self) -> Mesh {
        create_cylinder_mesh(
            self.radius(),
            self.height(),
            self.resolution() as u32,
            self.segments() as u32,
            self.cap(),
        )
    }
    fn to_node(&self) -> crate::bindings::exports::unavi::shapes::api::Node {
        let node = Node::new();
        node.set_mesh(Some(&self.to_mesh()));
        node
    }
    fn to_physics_node(&self) -> Node {
        let node = self.to_node();
        node.set_collider(Some(&Collider::new(Shape::Cylinder(ShapeCylinder {
            height: self.height(),
            radius: self.radius(),
        }))));
        node
    }
}

fn create_cylinder_mesh(
    radius: f32,
    height: f32,
    resolution: u32,
    segments: u32,
    cap: bool,
) -> Mesh {
    debug_assert!(resolution > 2);
    debug_assert!(segments > 0);

    let half_height = height / 2.0;

    let num_rings = segments + 1;
    let num_vertices = resolution * 2 + num_rings * (resolution + 1);
    let num_faces = resolution * (num_rings - 2);
    let num_indices = (2 * num_faces + 2 * (resolution - 1) * 2) * 3;

    let mut positions = Vec::with_capacity(num_vertices as usize);
    let mut normals = Vec::with_capacity(num_vertices as usize);
    let mut uvs = Vec::with_capacity(num_vertices as usize);
    let mut indices = Vec::with_capacity(num_indices as usize);

    let step_theta = std::f32::consts::TAU / resolution as f32;
    let step_y = 2.0 * half_height / segments as f32;

    // rings

    for ring in 0..num_rings {
        let y = -half_height + ring as f32 * step_y;

        for segment in 0..=resolution {
            let theta = segment as f32 * step_theta;
            let (sin, cos) = theta.sin_cos();

            positions.push([radius * cos, y, radius * sin]);
            normals.push([cos, 0., sin]);
            uvs.push([
                segment as f32 / resolution as f32,
                ring as f32 / segments as f32,
            ]);
        }
    }

    // barrel skin
    for i in 0..segments {
        let ring = i * (resolution + 1);
        let next_ring = (i + 1) * (resolution + 1);

        for j in 0..resolution {
            indices.extend_from_slice(&[
                ring + j,
                next_ring + j,
                ring + j + 1,
                next_ring + j,
                next_ring + j + 1,
                ring + j + 1,
            ]);
        }
    }

    // caps
    if cap {
        let mut build_cap = |top: bool| {
            let offset = positions.len() as u32;
            let (y, normal_y, winding) = if top {
                (half_height, 1., (1, 0))
            } else {
                (-half_height, -1., (0, 1))
            };

            for i in 0..resolution {
                let theta = i as f32 * step_theta;
                let (sin, cos) = theta.sin_cos();

                positions.push([cos * radius, y, sin * radius]);
                normals.push([0.0, normal_y, 0.0]);
                uvs.push([0.5 * (cos + 1.0), 1.0 - 0.5 * (sin + 1.0)]);
            }

            for i in 1..(resolution - 1) {
                indices.extend_from_slice(&[
                    offset,
                    offset + i + winding.0,
                    offset + i + winding.1,
                ]);
            }
        };

        build_cap(true);
        build_cap(false);
    }

    let normals = normals.into_iter().flatten().collect::<Vec<_>>();
    let positions = positions.into_iter().flatten().collect::<Vec<_>>();
    let uvs = uvs.into_iter().flatten().collect::<Vec<_>>();

    let mesh = Mesh::new();
    let primitive = mesh.create_primitive();

    primitive.set_indices(&indices);
    primitive.set_normals(&normals);
    primitive.set_positions(&positions);
    primitive.set_uvs(&uvs);

    mesh
}
