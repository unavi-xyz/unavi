use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::bindings::{
    exports::unavi::vscreen::screen::{
        ChildLayout, Container, GuestScreen, Screen as ScreenExport, ScreenBorrow, ScreenShape,
    },
    unavi::shapes::api::{Circle, Rectangle},
    wired::math::types::{Transform, Vec3},
};

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
    open_duration: Cell<f32>,
    root: Container,
    visible: Cell<bool>,
    visible_animating: Cell<bool>,
}

impl GuestScreen for Screen {
    fn new(shape: ScreenShape) -> Self {
        let (size, mesh) = match shape {
            ScreenShape::Circle(radius) => (
                Vec3::new(radius, radius, 0.0),
                Circle::new(radius).to_physics_node(),
            ),
            ScreenShape::Rectangle(size) => (
                Vec3::new(size.x, size.y, 0.0),
                Rectangle::new(size).to_physics_node(),
            ),
        };

        let root = Container::new(size);
        root.inner().add_child(&mesh);
        root.inner().set_transform(Transform {
            scale: Vec3::splat(MIN_SCALE),
            ..Default::default()
        });

        let id = ID.fetch_add(1, Ordering::Relaxed);

        Self(Rc::new(ScreenData {
            id,
            child_layout: Cell::new(ChildLayout::Butterfly),
            children: RefCell::default(),
            open_duration: Cell::new(1.0),
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
        self.0.visible.set(value);
        self.0.visible_animating.set(true);
    }

    fn open_duration(&self) -> f32 {
        self.0.open_duration.get()
    }
    fn set_open_duration(&self, value: f32) {
        self.0.open_duration.set(value);
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
            let mut transform = self.0.root.inner().transform();

            if visible {
                transform.scale += delta / self.0.open_duration.get();

                if transform.scale.x >= MAX_SCALE {
                    transform.scale = Vec3::splat(MAX_SCALE);
                    self.0.visible_animating.set(false);
                }
            } else {
                transform.scale -= delta / self.0.open_duration.get();

                if transform.scale.x <= MIN_SCALE {
                    transform.scale = Vec3::splat(MIN_SCALE);
                    self.0.visible_animating.set(false);
                }
            }

            self.0.root.inner().set_transform(transform);
        }

        if !visible {
            return;
        }

        for child in self.0.children.borrow().iter() {
            child.update(delta);
        }
    }
}
