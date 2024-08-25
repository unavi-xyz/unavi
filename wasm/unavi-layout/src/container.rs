use std::cell::{Cell, RefCell};

use crate::{
    bindings::{
        exports::unavi::layout::container::{Alignment, Guest, GuestContainer},
        wired::{math::types::Vec3, scene::node::Node},
    },
    GuestImpl,
};

impl Guest for GuestImpl {
    type Container = Container;
}

pub struct Container {
    align_x: Cell<Alignment>,
    align_y: Cell<Alignment>,
    align_z: Cell<Alignment>,
    root: Node,
    size: RefCell<Vec3>,
}

impl GuestContainer for Container {
    fn new(size: Vec3) -> Self {
        Self {
            align_x: Cell::new(Alignment::Center),
            align_y: Cell::new(Alignment::Center),
            align_z: Cell::new(Alignment::Center),
            root: Node::new(),
            size: RefCell::new(size),
        }
    }

    fn root(&self) -> crate::bindings::exports::unavi::layout::container::Node {
        todo!()
    }

    fn inner(&self) -> Node {
        todo!()
        // unsafe { Node::from_handle(self.node.handle()) }
    }

    fn size(&self) -> Vec3 {
        *self.size.borrow()
    }
    fn set_size(&self, value: Vec3) {
        self.size.replace(value);
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
