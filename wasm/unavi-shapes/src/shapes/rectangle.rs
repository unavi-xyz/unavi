use std::cell::RefCell;

use crate::bindings::{
    exports::unavi::shapes::api::GuestRectangle,
    wired::{
        math::types::{Vec2, Vec3},
        physics::types::{Collider, Shape},
        scene::{mesh::Mesh, node::Node},
    },
};

pub struct Rectangle {
    size: RefCell<Vec2>,
}

impl GuestRectangle for Rectangle {
    fn new(size: Vec2) -> Self {
        Self {
            size: RefCell::new(size),
        }
    }

    fn size(&self) -> Vec2 {
        *self.size.borrow()
    }
    fn set_size(&self, value: Vec2) {
        self.size.replace(value);
    }

    fn to_mesh(&self) -> Mesh {
        create_rectangle_mesh(self.size())
    }
    fn to_node(&self) -> crate::bindings::exports::unavi::shapes::api::Node {
        let node = Node::new();
        node.set_mesh(Some(&self.to_mesh()));
        node
    }
    fn to_physics_node(&self) -> Node {
        let node = self.to_node();
        let size = self.size();
        node.set_collider(Some(&Collider::new(Shape::Cuboid(Vec3::new(
            size.x, 0.0, size.y,
        )))));
        node
    }
}

fn create_rectangle_mesh(size: Vec2) -> Mesh {
    let normal = glam::Vec3::new(0.0, 0.0, -1.0);
    let subdivisions = 0;

    let z_vertex_count = subdivisions + 2;
    let x_vertex_count = subdivisions + 2;
    let num_vertices = (z_vertex_count * x_vertex_count) as usize;
    let num_indices = ((z_vertex_count - 1) * (x_vertex_count - 1) * 6) as usize;

    let mut positions: Vec<glam::Vec3> = Vec::with_capacity(num_vertices);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(num_vertices);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(num_vertices);
    let mut indices: Vec<u32> = Vec::with_capacity(num_indices);

    for z in 0..z_vertex_count {
        for x in 0..x_vertex_count {
            let tx = x as f32 / (x_vertex_count - 1) as f32;
            let tz = z as f32 / (z_vertex_count - 1) as f32;
            let pos = glam::Vec3::new((-0.5 + tx) * size.x, 0.0, (-0.5 + tz) * size.y);
            positions.push(pos);
            normals.push(normal.to_array());
            uvs.push([tx, tz]);
        }
    }

    for z in 0..z_vertex_count - 1 {
        for x in 0..x_vertex_count - 1 {
            let quad = z * x_vertex_count + x;
            indices.push(quad + x_vertex_count + 1);
            indices.push(quad + 1);
            indices.push(quad + x_vertex_count);
            indices.push(quad);
            indices.push(quad + x_vertex_count);
            indices.push(quad + 1);
        }
    }

    let normals = normals.into_iter().flatten().collect::<Vec<_>>();
    let positions = positions
        .into_iter()
        .flat_map(|v| [v.x, v.y, v.z])
        .collect::<Vec<_>>();
    let uvs = uvs.into_iter().flatten().collect::<Vec<_>>();

    let mesh = Mesh::new();
    let primitive = mesh.create_primitive();
    primitive.set_indices(&indices);
    primitive.set_normals(&normals);
    primitive.set_positions(&positions);
    primitive.set_uvs(&uvs);

    mesh
}
