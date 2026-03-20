use crate::{
    exports::wired::script::guest_api::{Guest, GuestScript},
    wired::input::{system_api::system_input_listener, types::InputListener},
};

wit_bindgen::generate!({
    generate_all,
});

struct World;

impl Guest for World {
    type Script = Script;
}

struct Script {
    input: InputListener,
}

impl GuestScript for Script {
    fn new() -> Self {
        let input = system_input_listener();
        Self { input }
    }

    fn tick(&self) {
        while let Some(event) = self.input.poll() {
            println!("{event:#?}");
        }
    }

    fn render(&self) {}

    fn drop(&self) {}
}

export!(World);
