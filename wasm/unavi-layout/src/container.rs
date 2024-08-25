use std::{cell::RefCell, rc::Rc};

use crate::{
    bindings::{
        exports::unavi::layout::container::{
            Alignment, Container as ContainerExport, Guest, GuestContainer,
        },
        wired::{math::types::Vec3, scene::node::Node},
    },
    GuestImpl,
};

impl Guest for GuestImpl {
    type Container = Container;
}

#[derive(Clone)]
pub struct Container(Rc<RefCell<ContainerData>>);

pub struct ContainerData {
    align_x: Alignment,
    align_y: Alignment,
    align_z: Alignment,
    inner: Node,
    root: Node,
    size: Vec3,
}

macro_rules! generate_align_methods {
    ($align_fn:ident, $set_align_fn:ident, $axis:ident) => {
        fn $align_fn(&self) -> Alignment {
            self.0.borrow().$align_fn
        }

        fn $set_align_fn(&self, value: Alignment) {
            let mut data = self.0.borrow_mut();

            data.$align_fn = value;

            let percent = match value {
                Alignment::Start => 0.0,
                Alignment::Center => 0.5,
                Alignment::End => 1.0,
            };

            let len = self.size().$axis;
            let axis_val = (percent * len) - (len / 2.0);

            let mut transform = data.inner.transform();
            transform.translation.$axis = axis_val;
            data.inner.set_transform(transform);
        }
    };
}

impl GuestContainer for Container {
    fn new(size: Vec3) -> Self {
        let inner = Node::new();
        let root = Node::new();
        root.add_child(&inner);

        Self(Rc::new(RefCell::new(ContainerData {
            align_x: Alignment::Center,
            align_y: Alignment::Center,
            align_z: Alignment::Center,
            inner,
            root,
            size,
        })))
    }

    fn ref_(&self) -> ContainerExport {
        ContainerExport::new(self.clone())
    }

    fn root(&self) -> Node {
        self.0.borrow().root.ref_()
    }

    fn inner(&self) -> Node {
        self.0.borrow().inner.ref_()
    }

    fn size(&self) -> Vec3 {
        self.0.borrow().size
    }
    fn set_size(&self, value: Vec3) {
        self.0.borrow_mut().size = value;

        // Use alignment methods to update inner node.
        self.set_align_x(self.align_x());
        self.set_align_y(self.align_y());
        self.set_align_z(self.align_z());
    }

    generate_align_methods!(align_x, set_align_x, x);
    generate_align_methods!(align_y, set_align_y, y);
    generate_align_methods!(align_z, set_align_z, z);
}
