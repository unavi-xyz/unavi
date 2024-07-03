use bindings::exports::wired::script::types::{Guest, GuestScript};

#[allow(warnings)]
mod bindings;

#[derive(Default)]
struct Script {}

impl GuestScript for Script {
    fn new() -> Self {
        Script::default()
    }

    fn update(&self, _delta: f32) {}
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
