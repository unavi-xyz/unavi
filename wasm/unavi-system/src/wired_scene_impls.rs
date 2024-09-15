#![allow(dead_code)]

use crate::bindings::wired::scene::{
    gltf::Scene,
    glxf::{GlxfNode, GlxfScene},
    material::{Color, Material},
    mesh::{Mesh, Primitive},
    node::Node,
};

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

impl PartialEq for GlxfNode {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialEq for GlxfScene {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

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
