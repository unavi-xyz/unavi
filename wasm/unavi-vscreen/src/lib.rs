use std::cell::{Cell, RefCell};

use bindings::{
    exports::unavi::vscreen::screen::{Guest, GuestModule, GuestScreen, Module as ModuleRes},
    wired::scene::node::Node,
};

#[allow(warnings)]
mod bindings;

struct Screen {
    modules: RefCell<Vec<ModuleRes>>,
    visible: Cell<bool>,
}

impl GuestScreen for Screen {
    fn new() -> Self {
        Self {
            modules: RefCell::default(),
            visible: Cell::new(false),
        }
    }

    fn add_module(&self, module: ModuleRes) {
        let mut modules = self.modules.borrow_mut();
        modules.push(module);
    }

    fn visible(&self) -> bool {
        self.visible.get()
    }
    fn set_visible(&self, value: bool) {
        self.visible.set(value);
    }

    fn update(&self, delta: f32) {
        if !self.visible.get() {
            return;
        }

        for module in self.modules.borrow().iter() {
            let module = module.get::<Module>();
            module.update(delta);
        }
    }
}

struct Module {
    root: Node,
}

impl GuestModule for Module {
    fn new() -> Self {
        let root = Node::new();

        Self { root }
    }

    fn root(&self) -> Node {
        todo!();
    }

    fn update(&self, _delta: f32) {}
}

struct Types;

impl Guest for Types {
    type Screen = Screen;
    type Module = Module;
}

bindings::export!(Types with_types_in bindings);
