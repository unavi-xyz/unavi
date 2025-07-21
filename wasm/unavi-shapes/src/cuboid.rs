use std::cell::Cell;

use crate::{
    exports::unavi::shapes::shapes::GuestCuboid,
    wired::{
        math::types::Vec3,
        scene::{mesh::Mesh, node::Node},
    },
};

pub struct Cuboid {
    size: Cell<Vec3>,
}

impl GuestCuboid for Cuboid {
    fn new(size: Vec3) -> Self {
        Self {
            size: Cell::new(size),
        }
    }

    fn size(&self) -> Vec3 {
        self.size.get()
    }
    fn set_size(&self, value: Vec3) {
        self.size.set(value);
    }

    fn to_mesh(&self) -> Mesh {
        todo!();
    }
    fn to_node(&self) -> Node {
        todo!();
    }
    fn to_physics_node(&self) -> Node {
        todo!();
    }
}
