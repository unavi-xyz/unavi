use std::cell::{Cell, RefCell};

use crate::{
    bindings::{
        exports::unavi::vscreen::screen::{
            GuestModule, GuestScreen, Module as ModuleExport, ModuleBorrow,
        },
        unavi::shapes::api::Cylinder,
        wired::{
            math::types::{Quat, Transform, Vec3},
            scene::node::Node,
        },
    },
    module::Module,
};

const ANIMATION_DURATION_SECONDS: f32 = 0.25;
const ARM_RADIUS: f32 = 0.03; // TODO: Get from avatar.
const SCREEN_HEIGHT: f32 = 0.002;
const SCREEN_RADIUS: f32 = 0.04;

pub struct Screen {
    central_module: RefCell<Option<Module>>,
    modules: RefCell<Vec<Module>>,
    root: Node,
    visible: Cell<bool>,
    visible_animating: Cell<bool>,
}

impl GuestScreen for Screen {
    fn new() -> Self {
        let root = Cylinder::new(SCREEN_RADIUS, SCREEN_HEIGHT).to_node();
        root.set_transform(Transform {
            translation: Vec3::new(SCREEN_RADIUS * 2.0, ARM_RADIUS, -ARM_RADIUS / 3.0),
            rotation: Quat::default(),
            scale: Vec3::default(),
        });

        Self {
            central_module: RefCell::default(),
            modules: RefCell::default(),
            root,
            visible: Cell::default(),
            visible_animating: Cell::default(),
        }
    }

    fn root(&self) -> Node {
        self.root.ref_()
    }

    fn visible(&self) -> bool {
        self.visible.get()
    }
    fn set_visible(&self, value: bool) {
        if self.visible.get() == value {
            return;
        }

        self.visible.set(value);
        self.visible_animating.set(true);
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
        let visible = self.visible.get();

        if self.visible_animating.get() {
            let mut transform = self.root.transform();

            if visible {
                transform.scale += delta / ANIMATION_DURATION_SECONDS;

                if transform.scale.x > 1.0 {
                    transform.scale = Vec3::splat(1.0);
                    self.visible_animating.set(false);
                }
            } else {
                transform.scale -= delta / ANIMATION_DURATION_SECONDS;

                if transform.scale.x < 0.0 {
                    transform.scale = Vec3::splat(0.0);
                    self.visible_animating.set(false);
                }
            }

            self.root.set_transform(transform);
        }

        if !visible {
            return;
        }

        for module in self.modules.borrow().iter() {
            module.update(delta);
        }
    }
}
