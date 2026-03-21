use crate::wired::input::{system_api::system_input_listener, types::InputListener};

wired_prelude::generate_script!(Script);

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
