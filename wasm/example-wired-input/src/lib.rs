use crate::wired::input::{
    context::register_global_input_listener,
    types::InputListener,
};

wired_prelude::generate_script!(Script);

struct Script {
    input: InputListener,
}

impl ScriptBehavior for Script {
    fn init() -> Self {
        let input = register_global_input_listener();
        Self { input }
    }

    fn tick(&mut self) {
        while let Some(event) = self.input.poll() {
            println!("got input: {event:#?}");
        }
    }
}
