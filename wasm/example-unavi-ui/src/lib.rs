use bindings::{
    exports::wired::script::types::{Guest, GuestScript},
    unavi::{
        scene::api::{Root, Scene},
        ui::button::Button,
    },
};

#[allow(warnings)]
mod bindings;

struct Script {
    button: Button,
}

impl GuestScript for Script {
    fn new() -> Self {
        let scene = Scene::new();

        let button = Button::new();
        // scene.add_node(&button.root().root());

        Root::add_scene(&scene);

        Script { button }
    }

    fn update(&self, _delta: f32) {}
}

struct Types;

impl Guest for Types {
    type Script = Script;
}

bindings::export!(Types with_types_in bindings);
