use bindings::exports::wired::script::types::{Guest, GuestScript};

#[allow(warnings)]
mod bindings;
mod impls;

#[derive(Default)]
struct Script {}

impl GuestScript for Script {
    fn new() -> Self {
        Script::default()
    }

    fn update(&self, _delta: f32) {}
}

struct Api;

impl Guest for Api {
    type Script = Script;
}

bindings::export!(Api with_types_in bindings);
