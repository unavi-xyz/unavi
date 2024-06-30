#![allow(dead_code)]

use crate::bindings::wired::scene::{
    material::Material,
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
