use std::{cell::Cell, f32::consts::FRAC_PI_2};

use crate::bindings::{
    exports::unavi::shapes::api::{GuestCircle, GuestEllipse},
    wired::{
        math::types::Vec2,
        physics::types::{Collider, Shape, ShapeCylinder},
        scene::{mesh::Mesh, node::Node},
    },
};

use super::ellipse::Ellipse;

pub struct Circle {
    radius: Cell<f32>,
    resolution: Cell<u16>,
}

impl Circle {
    fn to_ellipse(&self) -> Ellipse {
        let radius = self.radius.get();
        let resolution = self.resolution.get();
        let ellipse = Ellipse::new(Vec2::splat(radius));
        ellipse.set_resolution(resolution);
        ellipse
    }
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
        self.to_ellipse().to_mesh()
    }
    fn to_node(&self) -> crate::bindings::exports::unavi::shapes::api::Node {
        self.to_ellipse().to_node()
    }
    fn to_physics_node(&self) -> Node {
        let node = self.to_ellipse().to_node();
        node.set_collider(Some(&Collider::new(Shape::Cylinder(ShapeCylinder {
            radius: self.radius.get(),
            height: 0.0,
        }))));
        node
    }
}
