use std::{cell::Cell, rc::Rc, sync::atomic::Ordering};

use crate::{
    bindings::{
        exports::unavi::ui::button::{Guest, GuestButton},
        unavi::{layout::container::Container, shapes::api::Cuboid},
        wired::{
            input::{handler::InputHandler, types::InputAction},
            scene::material::{Color, Material},
        },
    },
    GuestImpl, Updatable, ELEMENTS, ELEMENT_ID,
};

impl Guest for GuestImpl {
    type Button = Button;
}

pub struct Button(Rc<ButtonData>);

impl Drop for Button {
    fn drop(&mut self) {
        self.0.remove_element();
    }
}

pub struct ButtonData {
    id: usize,

    color: Color,
    color_hover: Color,

    input: InputHandler,
    material: Material,
    root: Container,

    hovered: Cell<bool>,
    pressed: Cell<bool>,
}

impl Updatable for ButtonData {
    fn id(&self) -> usize {
        self.id
    }

    fn update(&self, _delta: f32) {
        // Handle input.
        self.hovered.set(false);
        self.pressed.set(false);

        while let Some(event) = self.input.next() {
            match event.action {
                InputAction::Hover => self.hovered.set(true),
                InputAction::Collision => self.pressed.set(true),
            }
        }

        // Animation.
        if self.hovered.get() {
            self.material.set_color(self.color_hover);
        } else {
            self.material.set_color(self.color);
        }
    }
}

const DARKEN_PERCENT: f32 = 0.5;

impl GuestButton for Button {
    fn new(root: Container) -> Self {
        let size = root.size();
        let node = Cuboid::new(size).to_physics_node();

        let input = InputHandler::new();
        node.set_input_handler(Some(&input));

        let material = Material::new();
        for primitive in node.mesh().unwrap().list_primitives() {
            primitive.set_material(Some(&material));
        }

        root.inner().add_child(&node);

        let color = Color {
            r: 0.2,
            g: 0.7,
            b: 1.0,
            a: 1.0,
        };

        let color_hover = Color {
            r: color.r * DARKEN_PERCENT,
            g: color.g * DARKEN_PERCENT,
            b: color.b * DARKEN_PERCENT,
            a: color.a,
        };

        let data = Rc::new(ButtonData {
            id: ELEMENT_ID.fetch_add(1, Ordering::Relaxed),

            color,
            color_hover,

            input,
            material,
            root,

            hovered: Cell::default(),
            pressed: Cell::default(),
        });

        unsafe {
            ELEMENTS.borrow_mut().push(data.clone());
        }

        Self(data)
    }

    fn root(&self) -> Container {
        self.0.root.ref_()
    }

    fn hovered(&self) -> bool {
        self.0.hovered.get()
    }
    fn pressed(&self) -> bool {
        self.0.pressed.get()
    }
}
