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

struct Script;

impl GuestScript for Script {
    fn new() -> Self {
        println!("initializing vscreen");

        let _doc = self_document();

        let _base = Cylinder::new(0.2, 0.02);

        Self
    }

    fn tick(&self) {}

    fn render(&self) {}

    fn drop(&self) {}
}

export!(World);
