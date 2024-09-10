use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::bindings::{
    exports::unavi::vscreen::screen::{
        ChildLayout, Container, GuestScreen, Screen as ScreenExport, ScreenBorrow, ScreenShape,
    },
    unavi::shapes::api::{Cylinder, Rectangle},
    wired::math::types::{Transform, Vec3},
};

const ANIMATION_DURATION_SECONDS: f32 = 0.3;
const ARM_RADIUS: f32 = 0.03;
const MAX_SCALE: f32 = 1.0;
const MIN_SCALE: f32 = 0.0;

#[derive(Clone)]
pub struct Screen(pub Rc<ScreenData>);

impl PartialEq for Screen {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}

static ID: AtomicUsize = AtomicUsize::new(0);

pub struct ScreenData {
    id: usize,
    child_layout: Cell<ChildLayout>,
    children: RefCell<Vec<Screen>>,
    root: Container,
    visible: Cell<bool>,
    visible_animating: Cell<bool>,
}

impl GuestScreen for Screen {
    fn new(shape: ScreenShape) -> Self {
        let (size, mesh) = match shape {
            ScreenShape::Circle(radius) => {
                let mesh = Cylinder::new(radius, 0.005).to_physics_node();
                mesh.set_transform(Transform {
                    translation: Vec3::new(radius * 2.0, ARM_RADIUS, -ARM_RADIUS / 3.0),
                    scale: Vec3::splat(MIN_SCALE),
                    ..Default::default()
                });
                (Vec3::new(radius, radius, 0.01), mesh)
            }
            ScreenShape::Rectangle(size) => {
                let mesh = Rectangle::new(size).to_physics_node();
                mesh.set_transform(Transform {
                    translation: Vec3::new(size.x, ARM_RADIUS, -ARM_RADIUS / 3.0),
                    scale: Vec3::splat(MIN_SCALE),
                    ..Default::default()
                });
                (Vec3::new(size.x, size.y, 0.01), mesh)
            }
        };

        let root = Container::new(size);
        root.inner().add_child(&mesh);

        let id = ID.fetch_add(1, Ordering::Relaxed);

        Self(Rc::new(ScreenData {
            id,
            child_layout: Cell::new(ChildLayout::Butterfly),
            children: RefCell::default(),
            root,
            visible: Cell::default(),
            visible_animating: Cell::default(),
        }))
    }

    fn root(&self) -> Container {
        self.0.root.ref_()
    }

    fn visible(&self) -> bool {
        self.0.visible.get()
    }
    fn set_visible(&self, value: bool) {
        if self.0.visible.get() == value {
            return;
        }

        self.0.visible.set(value);
        self.0.visible_animating.set(true);
    }

    fn child_layout(&self) -> ChildLayout {
        self.0.child_layout.get()
    }
    fn set_child_layout(&self, value: ChildLayout) {
        self.0.child_layout.set(value);
    }

    fn children(&self) -> Vec<ScreenExport> {
        self.0
            .children
            .borrow()
            .iter()
            .cloned()
            .map(ScreenExport::new)
            .collect()
    }
    fn add_child(&self, value: ScreenBorrow) {
        let value = value.get::<Screen>();
        self.0.root.add_child(&value.0.root);
        self.0.children.borrow_mut().push(value.clone());
    }
    fn remove_child(&self, value: ScreenBorrow) {
        let value = value.get::<Screen>();
        self.0.root.remove_child(&value.0.root);
        self.0
            .children
            .borrow()
            .iter()
            .position(|n| n == value)
            .map(|index| self.0.children.borrow_mut().remove(index));
    }

    fn update(&self, delta: f32) {
        let visible = self.0.visible.get();

        if self.0.visible_animating.get() {
            let mut transform = self.0.root.root().transform();

            if visible {
                transform.scale += delta / ANIMATION_DURATION_SECONDS;

                if transform.scale.x > MAX_SCALE {
                    transform.scale = Vec3::splat(MAX_SCALE);
                    self.0.visible_animating.set(false);
                }
            } else {
                transform.scale -= delta / ANIMATION_DURATION_SECONDS;

                if transform.scale.x < MIN_SCALE {
                    transform.scale = Vec3::splat(MIN_SCALE);
                    self.0.visible_animating.set(false);
                }
            }

            self.0.root.root().set_transform(transform);
        }

        if !visible {
            return;
        }

        for module in self.0.children.borrow().iter() {
            module.update(delta);
        }
    }
}
