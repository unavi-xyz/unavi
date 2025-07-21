use exports::wired::script::types::{Guest, GuestScript};

wit_bindgen::generate!({ generate_all });

struct World;

impl Guest for World {
    type Script = Script;
}

struct Script;

impl GuestScript for Script {
    fn new() -> Self {
        println!("new");
        Self
    }

    fn update(&self, _delta: f32) {
        println!("update");
    }

    fn render(&self, _delta: f32) {
        println!("render");
    }
}

export!(World);
