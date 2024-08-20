use std::cell::Cell;

use crate::bindings::{
    exports::unavi::shapes::api::GuestCylinder,
    wired::{
        physics::types::{Collider, Shape, ShapeCylinder},
        scene::{mesh::Mesh, node::Node},
    },
};

pub struct Cylinder {
    radius: Cell<f32>,
    height: Cell<f32>,
}

impl GuestCylinder for Cylinder {
    fn new(radius: f32, height: f32) -> Self {
        Self {
            radius: Cell::new(radius),
            height: Cell::new(height),
        }
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

    fn to_mesh(&self) -> Mesh {
        let mesh = Mesh::new();
        let primitive = mesh.create_primitive();

        mesh
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
