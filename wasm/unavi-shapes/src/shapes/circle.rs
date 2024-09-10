use std::cell::Cell;

use crate::bindings::{
    exports::unavi::shapes::api::GuestCircle,
    wired::{
        math::types::Vec2,
        physics::types::{Collider, Shape, ShapeCylinder},
        scene::{mesh::Mesh, node::Node},
    },
};

use super::ellipse::create_ellipse_mesh;

pub struct Circle {
    radius: Cell<f32>,
    resolution: Cell<u16>,
}

impl GuestCircle for Circle {
    fn new(radius: f32) -> Self {
        Self {
            radius: Cell::new(radius),
            resolution: Cell::new(32),
        }
    }

    fn radius(&self) -> f32 {
        self.radius.get()
    }
    fn set_radius(&self, value: f32) {
        self.radius.set(value);
    }

    fn resolution(&self) -> u16 {
        self.resolution.get()
    }
    fn set_resolution(&self, value: u16) {
        self.resolution.set(value);
    }

    fn to_mesh(&self) -> Mesh {
        let radius = self.radius.get();
        let resolution = self.resolution.get();
        create_ellipse_mesh(Vec2::splat(radius), resolution)
    }
    fn to_node(&self) -> crate::bindings::exports::unavi::shapes::api::Node {
        let node = Node::new();
        node.set_mesh(Some(&self.to_mesh()));
        node
    }
    fn to_physics_node(&self) -> Node {
        let node = self.to_node();
        let radius = self.radius.get();
        node.set_collider(Some(&Collider::new(Shape::Cylinder(ShapeCylinder {
            radius,
            height: 0.0,
        }))));
        node
    }
}
