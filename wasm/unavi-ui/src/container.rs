use std::cell::Cell;

use crate::{
    bindings::{
        exports::unavi::ui::container::{Alignment, Guest, GuestContainer},
        wired::scene::node::{create_node, Node},
    },
    GuestImpl,
};

impl Guest for GuestImpl {
    type Container = Container;
}

pub struct Container {
    node: Node,
    x_len: Cell<f32>,
    y_len: Cell<f32>,
    z_len: Cell<f32>,
    align_x: Cell<Alignment>,
    align_y: Cell<Alignment>,
    align_z: Cell<Alignment>,
}

impl GuestContainer for Container {
    fn new() -> Self {
        Self {
            node: create_node(),
            x_len: Cell::new(1.0),
            y_len: Cell::new(1.0),
            z_len: Cell::new(1.0),
            align_x: Cell::new(Alignment::Center),
            align_y: Cell::new(Alignment::Center),
            align_z: Cell::new(Alignment::Center),
        }
    }

    fn root(&self) -> Node {
        // self.node
        todo!()
    }

    fn x_len(&self) -> f32 {
        self.x_len.get()
    }
    fn y_len(&self) -> f32 {
        self.y_len.get()
    }
    fn z_len(&self) -> f32 {
        self.z_len.get()
    }
    fn set_x_len(&self, value: f32) {
        self.x_len.set(value);
    }
    fn set_y_len(&self, value: f32) {
        self.y_len.set(value);
    }
    fn set_z_len(&self, value: f32) {
        self.z_len.set(value);
    }

    fn align_x(&self) -> Alignment {
        self.align_x.get()
    }
    fn align_y(&self) -> Alignment {
        self.align_y.get()
    }
    fn align_z(&self) -> Alignment {
        self.align_z.get()
    }
    fn set_align_x(&self, value: Alignment) {
        self.align_x.set(value);
    }
    fn set_align_y(&self, value: Alignment) {
        self.align_y.set(value);
    }
    fn set_align_z(&self, value: Alignment) {
        self.align_z.set(value);
    }
}
