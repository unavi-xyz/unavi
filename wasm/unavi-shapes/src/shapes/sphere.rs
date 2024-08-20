use std::cell::Cell;

use crate::bindings::{
    exports::unavi::shapes::api::GuestSphere,
    wired::{
        physics::types::{Collider, Shape},
        scene::{mesh::Mesh, node::Node},
    },
};

pub struct Sphere {
    radius: Cell<f32>,
    sectors: Cell<u16>,
    stacks: Cell<u16>,
}

impl GuestSphere for Sphere {
    fn new(radius: f32) -> Self {
        Self {
            radius: Cell::new(radius),
            sectors: Cell::new(32),
            stacks: Cell::new(18),
        }
    }

    fn radius(&self) -> f32 {
        self.radius.get()
    }
    fn set_radius(&self, value: f32) {
        self.radius.set(value);
    }

    fn sectors(&self) -> u16 {
        self.sectors.get()
    }
    fn set_sectors(&self, value: u16) {
        self.sectors.set(value);
    }

    fn stacks(&self) -> u16 {
        self.stacks.get()
    }
    fn set_stacks(&self, value: u16) {
        self.stacks.set(value);
    }

    fn to_mesh(&self) -> Mesh {
        // TODO: make sphere
        Mesh::new()
    }
    fn to_node(&self) -> crate::bindings::exports::unavi::shapes::api::Node {
        let node = Node::new();
        node.set_mesh(Some(&self.to_mesh()));
        node
    }
    fn to_physics_node(&self) -> Node {
        let node = self.to_node();
        node.set_collider(Some(&Collider::new(Shape::Sphere(self.radius()))));
        node
    }
}
