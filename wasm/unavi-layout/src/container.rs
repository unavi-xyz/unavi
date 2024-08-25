use std::{cell::RefCell, rc::Rc};

use crate::{
    bindings::{
        exports::unavi::layout::container::{
            Alignment, Container as ContainerExport, ContainerBorrow, Guest, GuestContainer,
        },
        wired::{
            math::types::{Transform, Vec3},
            scene::node::Node,
        },
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
    children: Vec<Container>,
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

            let len = data.size.$axis;
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
            children: Vec::default(),
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

    fn list_children(&self) -> Vec<ContainerExport> {
        self.0
            .borrow()
            .children
            .iter()
            .cloned()
            .map(ContainerExport::new)
            .collect()
    }
    fn add_child(&self, child: ContainerBorrow) {
        let child = child.get::<Container>();
        self.0.borrow_mut().children.push(child.clone());

        self.inner().add_child(&child.root());

        let data = self.0.borrow();
        let size = child.size();

        let side = match data.align_x {
            Alignment::Start => 1.0,
            Alignment::Center => 0.0,
            Alignment::End => -1.0,
        };
        let offset_x = side * size.x / 2.0;

        let side = match data.align_y {
            Alignment::Start => 1.0,
            Alignment::Center => 0.0,
            Alignment::End => -1.0,
        };
        let offset_y = side * size.y / 2.0;

        let side = match data.align_z {
            Alignment::Start => 1.0,
            Alignment::Center => 0.0,
            Alignment::End => -1.0,
        };
        let offset_z = side * size.z / 2.0;

        child
            .root()
            .set_transform(Transform::from_translation(Vec3::new(
                offset_x, offset_y, offset_z,
            )));
    }
    fn remove_child(&self, child: ContainerBorrow) {
        let child = child.get::<Container>();
        self.0.borrow_mut().children.push(child.clone());

        self.inner().remove_child(&child.root());
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
