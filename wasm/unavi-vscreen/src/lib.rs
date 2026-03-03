use std::{cell::Cell, time::SystemTime};

use crate::{
    exports::wired::script::guest_api::{Guest, GuestScript},
    unavi::shapes::api::Cylinder,
    wired::scene::context::self_document,
};

wit_bindgen::generate!({
    generate_all,
});

struct World;

impl Guest for World {
    type Script = Script;
}

struct Script {
    pub time: Cell<SystemTime>,
}

impl GuestScript for Script {
    fn new() -> Self {
        println!("initializing vscreen");

        let _doc = self_document();
        let _base = Cylinder::new(0.2, 0.02);

        Self {
            time: Cell::new(SystemTime::now()),
        }
    }

    fn tick(&self) {
        let delta = self.time.get().elapsed().expect("elapsed").as_secs_f32();
        self.time.set(SystemTime::now());
        println!("<delta> {delta:.4}s");
    }

    fn render(&self) {}

    fn drop(&self) {}
}

export!(World);
