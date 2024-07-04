use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::ui::button::Button,
};

#[allow(warnings)]
mod bindings;

struct Script {
    button: Button,
}

impl GuestScript for Script {
    fn new() -> Self {
        let button = Button::new();

        Script { button }
    }

    fn update(&self, _delta: f32) {}
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
