#![allow(dead_code)]

use crate::bindings::wired::scene::{
    material::{Color, Material},
    mesh::{Mesh, Primitive},
    node::Node,
    scene::Scene,
};

impl Color {
    pub const BLACK: Color = Self::rgb(0.0, 0.0, 0.0);
    pub const WHITE: Color = Self::rgb(1.0, 1.0, 1.0);
    pub const RED: Color = Self::rgb(1.0, 0.2, 0.2);
    pub const BLUE: Color = Self::rgb(0.2, 0.2, 1.0);
    pub const GREEN: Color = Self::rgb(0.2, 1.0, 0.2);

    pub const fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b, a: 1.0 }
    }
}

impl Material {
    pub fn from_color(color: Color) -> Self {
        let material = Self::new();
        material.set_color(color);
        material
    }
}

impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq for Mesh {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq for Primitive {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq for Scene {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
