use std::{
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::bindings::{exports::unavi::vscreen::screen::GuestModule, wired::scene::node::Node};

#[derive(Clone)]
pub struct Module(pub Rc<ModuleData>);

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}

pub struct ModuleData {
    id: usize,
    root: Node,
}

static ID: AtomicUsize = AtomicUsize::new(0);

impl GuestModule for Module {
    fn new() -> Self {
        let id = ID.fetch_add(1, Ordering::Relaxed);
        let root = Node::new();

        Self(Rc::new(ModuleData { id, root }))
    }

    fn root(&self) -> Node {
        todo!();
    }

    fn update(&self, _delta: f32) {}
}
