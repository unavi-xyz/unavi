use crate::exports::wired::script::guest_api::{Guest, GuestScript};

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
        println!("hello from init");
        Self
    }

    fn tick(&self) {
        #[expect(clippy::empty_loop)]
        loop {
            // Do nothing
        }
    }

    fn render(&self) {}

    fn drop(&self) {}
}

export!(World);
