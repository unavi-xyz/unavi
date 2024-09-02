use std::cell::{Cell, RefCell};

use crate::{
    bindings::exports::unavi::vscreen::screen::{
        GuestModule, GuestScreen, Module as ModuleExport, ModuleBorrow,
    },
    module::Module,
};

pub struct Screen {
    central_module: RefCell<Option<Module>>,
    modules: RefCell<Vec<Module>>,
    visible: Cell<bool>,
}

impl GuestScreen for Screen {
    fn new() -> Self {
        Self {
            central_module: RefCell::default(),
            modules: RefCell::default(),
            visible: Cell::default(),
        }
    }

    fn visible(&self) -> bool {
        self.visible.get()
    }
    fn set_visible(&self, value: bool) {
        self.visible.set(value);
    }

    fn central_module(&self) -> Option<ModuleExport> {
        self.central_module.borrow().clone().map(ModuleExport::new)
    }
    fn set_central_module(&self, value: Option<ModuleBorrow>) {
        if let Some(value) = value {
            let module = value.get::<Module>();
            *self.central_module.borrow_mut() = Some(module.clone());
        } else {
            *self.central_module.borrow_mut() = None;
        }
    }

    fn add_module(&self, value: ModuleBorrow) {
        self.modules
            .borrow_mut()
            .push(value.get::<Module>().clone());
    }
    fn remove_module(&self, value: ModuleBorrow) {
        let value = value.get::<Module>();
        self.modules
            .borrow()
            .iter()
            .position(|n| n == value)
            .map(|index| self.modules.borrow_mut().remove(index));
    }
    fn modules(&self) -> Vec<ModuleExport> {
        self.modules
            .borrow()
            .iter()
            .cloned()
            .map(ModuleExport::new)
            .collect()
    }

    fn update(&self, delta: f32) {
        if !self.visible.get() {
            return;
        }

        for module in self.modules.borrow().iter() {
            module.update(delta);
        }
    }
}
