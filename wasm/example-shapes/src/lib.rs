use crate::{
    exports::wired::script::guest_api::{Guest, GuestScript},
    unavi::shapes::api::CuboidBuilder,
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
        let cube = CuboidBuilder::new();

        Self
    }

    fn tick(&self) {}

    fn render(&self) {}

    fn drop(&self) {}
}

export!(World);
