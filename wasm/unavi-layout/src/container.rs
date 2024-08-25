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
    inner: Node,
    root: Node,
    size: RefCell<Vec3>,
}

macro_rules! generate_align_methods {
    ($align_fn:ident, $set_align_fn:ident, $axis:ident) => {
        fn $align_fn(&self) -> Alignment {
            self.$align_fn.get()
        }

        fn $set_align_fn(&self, value: Alignment) {
            self.$align_fn.set(value);

            let percent = match value {
                Alignment::Start => 0.0,
                Alignment::Center => 0.5,
                Alignment::End => 1.0,
            };

            let len = self.size().$axis;
            let axis_val = (percent * len) - (len / 2.0);

            let mut transform = self.inner.transform();
            transform.translation.$axis = axis_val;
            self.inner.set_transform(transform);
        }
    };
}

impl GuestContainer for Container {
    fn new(size: Vec3) -> Self {
        let inner = Node::new();
        let root = Node::new();
        root.add_child(&inner);

        Self {
            align_x: Cell::new(Alignment::Center),
            align_y: Cell::new(Alignment::Center),
            align_z: Cell::new(Alignment::Center),
            inner,
            root,
            size: RefCell::new(size),
        }
    }

    fn root(&self) -> crate::bindings::exports::unavi::layout::container::Node {
        self.root.ref_()
    }

    fn inner(&self) -> Node {
        self.inner.ref_()
    }

    fn size(&self) -> Vec3 {
        *self.size.borrow()
    }
    fn set_size(&self, value: Vec3) {
        self.size.replace(value);

        // Use alignment methods to update inner node.
        self.set_align_x(self.align_x());
        self.set_align_y(self.align_y());
        self.set_align_z(self.align_z());
    }

    generate_align_methods!(align_x, set_align_x, x);
    generate_align_methods!(align_y, set_align_y, y);
    generate_align_methods!(align_z, set_align_z, z);
}
