use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::vscreen::screen::Screen,
};

#[allow(warnings)]
mod bindings;
mod wired_math_impls;
mod wired_scene_impls;

struct Script {
    screen: Screen,
}

impl GuestScript for Script {
    fn new() -> Self {
        Script {
            screen: Screen::new(),
        }
    }

    fn update(&self, delta: f32) {
        self.screen.update(delta);
    }
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
