use std::{
    cell::{Cell, RefCell},
    f32::consts::{FRAC_PI_2, PI},
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::bindings::{
    exports::unavi::vscreen::screen::{
        Container, GuestScreen, Screen as ScreenExport, ScreenBorrow, ScreenShape,
    },
    unavi::shapes::api::{Circle, Rectangle},
    wired::{
        input::{handler::InputHandler, types::InputAction},
        math::types::{Quat, Transform, Vec2, Vec3},
    },
};

const MAX_SCALE: f32 = 1.0;
const MIN_SCALE: f32 = 0.0;

static ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone)]
pub struct Screen(pub Rc<ScreenData>);

impl PartialEq for Screen {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}

pub struct ScreenData {
    animation_duration: Cell<f32>,
    children: RefCell<Vec<Screen>>,
    id: usize,
    input: InputHandler,
    open: Cell<bool>,
    root: Container,
    size: Vec2,
    visible: Cell<bool>,
    visible_animating: Cell<bool>,
}

impl Screen {
    fn position_children(&self, open: bool) {
        let children = self.0.children.borrow();
        let count = children.len();
        let angle_step = 2.0 * PI / count as f32;
        let radius = self.0.size.x.max(self.0.size.y) * 1.5;

        for (i, child) in children.iter().enumerate() {
            let angle = i as f32 * angle_step;

            let mut transform = child.root().inner().transform();
            transform.translation = Vec3::new(radius * angle.cos(), radius * angle.sin(), 0.0);
            child.root().inner().set_transform(transform);

            child.set_visible(open);
        }
    }
}

impl GuestScreen for Screen {
    fn new(shape: ScreenShape) -> Self {
        let (size, mesh) = match shape {
            ScreenShape::Circle(radius) => {
                let node = Circle::new(radius).to_physics_node();
                node.set_transform(Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2)));
                (Vec2::new(radius * 2.0, radius * 2.0), node)
            }
            ScreenShape::Rectangle(size) => (size, Rectangle::new(size).to_physics_node()),
        };

        let input = InputHandler::new();
        mesh.set_input_handler(Some(&input));

        let root = Container::new(Vec3::new(size.x, size.y, 0.0));
        root.inner().add_child(&mesh);
        root.inner().set_transform(Transform {
            scale: Vec3::splat(MIN_SCALE),
            ..Default::default()
        });

        let id = ID.fetch_add(1, Ordering::Relaxed);

        Self(Rc::new(ScreenData {
            animation_duration: Cell::new(0.5),
            children: RefCell::default(),
            id,
            input,
            root,
            size,
            open: Cell::default(),
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

    fn animation_duration(&self) -> f32 {
        self.0.animation_duration.get()
    }
    fn set_animation_duration(&self, value: f32) {
        self.0.animation_duration.set(value);
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
        self.position_children(self.0.open.get());
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
        self.position_children(self.0.open.get());
    }

    fn update(&self, delta: f32) {
        let open = self.0.open.get();
        let visible = self.0.visible.get();

        if self.0.visible_animating.get() {
            let mut transform = self.0.root.inner().transform();

            if visible {
                transform.scale += delta / self.0.animation_duration.get();

                if transform.scale.x >= MAX_SCALE {
                    transform.scale = Vec3::splat(MAX_SCALE);
                    self.0.visible_animating.set(false);
                }
            } else {
                transform.scale -= delta / self.0.animation_duration.get();

                if transform.scale.x <= MIN_SCALE {
                    transform.scale = Vec3::splat(MIN_SCALE);
                    self.0.visible_animating.set(false);
                }
            }

            self.0.root.inner().set_transform(transform);
        }

        while let Some(event) = self.0.input.next() {
            if event.action == InputAction::Collision {
                self.0.open.set(!open);
                self.position_children(!open);
            }
        }

        for child in self.0.children.borrow().iter() {
            child.update(delta);
        }
    }
}
