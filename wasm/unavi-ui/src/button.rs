use crate::{
    bindings::{
        exports::unavi::ui::button::{Guest, GuestButton},
        unavi::{layout::container::Container, shapes::api::Cuboid},
        wired::input::handler::InputHandler,
    },
    GuestImpl,
};

impl Guest for GuestImpl {
    type Button = Button;
}

pub struct Button {
    input: InputHandler,
    root: Container,
}

impl GuestButton for Button {
    fn new(root: Container) -> Self {
        let size = root.size();
        let node = Cuboid::new(size).to_physics_node();

        let input = InputHandler::new();
        node.set_input_handler(Some(&input));

        root.inner().add_child(&node);

        Self { input, root }
    }

    fn root(&self) -> Container {
        self.root.ref_()
    }

    fn pressed(&self) -> bool {
        self.input.handle_input().is_some()
    }
}
