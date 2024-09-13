use std::{cell::Cell, f32::consts::FRAC_PI_2};

use crate::bindings::{
    exports::unavi::shapes::api::GuestEllipse,
    wired::{
        math::types::{Quat, Transform, Vec2},
        scene::{mesh::Mesh, node::Node},
    },
};

pub struct Ellipse {
    half_size: Cell<Vec2>,
    resolution: Cell<u16>,
}

impl GuestEllipse for Ellipse {
    fn new(half_size: Vec2) -> Self {
        Self {
            half_size: Cell::new(half_size),
            resolution: Cell::new(32),
        }
    }

    fn half_size(&self) -> Vec2 {
        self.half_size.get()
    }
    fn set_half_size(&self, value: Vec2) {
        self.half_size.set(value);
    }

    fn resolution(&self) -> u16 {
        self.resolution.get()
    }
    fn set_resolution(&self, value: u16) {
        self.resolution.set(value);
    }

    fn to_mesh(&self) -> Mesh {
        let half_size = self.half_size.get();
        let resolution = self.resolution.get();
        create_ellipse_mesh(half_size, resolution)
    }
    fn to_node(&self) -> Node {
        let node = Node::new();
        node.set_mesh(Some(&self.to_mesh()));
        node
    }
    fn to_physics_node(&self) -> Node {
        // TODO: use mesh collider
        todo!()
    }
}

fn create_ellipse_mesh(half_size: Vec2, resolution: u16) -> Mesh {
    let resolution = resolution as usize;

    let mut indices = Vec::with_capacity((resolution - 2) * 3);
    let mut positions = Vec::with_capacity(resolution);
    let normals = vec![[0.0, -1.0, 0.0]; resolution];
    let mut uvs = Vec::with_capacity(resolution);

    // Add pi/2 so that there is a vertex at the top (sin is 1.0 and cos is 0.0)
    let start_angle = std::f32::consts::FRAC_PI_2;
    let step = std::f32::consts::TAU / resolution as f32;

    for i in 0..resolution {
        // Compute vertex position at angle theta
        let theta = start_angle + i as f32 * step;
        let (sin, cos) = theta.sin_cos();
        let x = cos * half_size.x;
        let z = sin * half_size.y;

        positions.push([x, 0.0, z]);
        uvs.push([0.5 * (cos + 1.0), 1.0 - 0.5 * (sin + 1.0)]);
    }

    for i in 1..(resolution as u32 - 1) {
        indices.extend_from_slice(&[0, i, i + 1]);
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
