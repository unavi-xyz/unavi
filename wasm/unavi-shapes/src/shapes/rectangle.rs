use std::{cell::RefCell, f32::consts::PI};

use crate::bindings::{
    exports::unavi::shapes::api::GuestRectangle,
    wired::{
        math::types::{Quat, Transform, Vec2, Vec3},
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
            size.x, size.y, 0.0,
        )))));
        node
    }
}

fn create_rectangle_mesh(size: Vec2) -> Mesh {
    let [hw, hh] = [size.x / 2.0, size.y / 2.0];
    let positions = vec![
        [hw, hh, 0.0],
        [-hw, hh, 0.0],
        [-hw, -hh, 0.0],
        [hw, -hh, 0.0],
    ];
    let normals = vec![[0.0, 0.0, 1.0]; 4];
    let uvs = vec![[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
    let indices = vec![0, 1, 2, 0, 2, 3];

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
