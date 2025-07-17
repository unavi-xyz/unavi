use exports::wired::script::types::{Guest, GuestScript};

wit_bindgen::generate!({ generate_all });

struct World;

impl Guest for World {
    type Script = Script;
}

struct Script;

impl GuestScript for Script {
    fn new() -> Self {
        println!("unavi:system new");
        Self
    }

    fn update(&self, delta: f32) {
        println!("unavi:system update: {delta:.4}");
    }

    fn render(&self, _delta: f32) {}
}

export!(World);
