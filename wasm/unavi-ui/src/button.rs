use std::{cell::Cell, rc::Rc, sync::atomic::Ordering};

use crate::{
    bindings::{
        exports::unavi::ui::button::{Guest, GuestButton},
        unavi::{layout::container::Container, shapes::api::Cuboid},
        wired::input::{handler::InputHandler, types::InputAction},
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
    input: InputHandler,
    root: Container,
    hovered: Cell<bool>,
    pressed: Cell<bool>,
}

impl Updatable for ButtonData {
    fn id(&self) -> usize {
        self.id
    }

    fn update(&self, _delta: f32) {
        self.hovered.set(false);
        self.pressed.set(false);

        while let Some(event) = self.input.next() {
            match event.action {
                InputAction::Hover => self.hovered.set(true),
                InputAction::Collision => self.pressed.set(true),
            }
        }
    }
}

impl GuestButton for Button {
    fn new(root: Container) -> Self {
        let size = root.size();
        let node = Cuboid::new(size).to_physics_node();

        let input = InputHandler::new();
        node.set_input_handler(Some(&input));

        root.inner().add_child(&node);

        let data = Rc::new(ButtonData {
            id: ELEMENT_ID.fetch_add(1, Ordering::Relaxed),
            input,
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
