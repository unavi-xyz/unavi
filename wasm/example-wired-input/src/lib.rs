use crate::wired::input::{
    context::register_global_input_listener,
    types::InputListener,
};

wired_prelude::generate_script!(Script);

struct Script {
    input: InputListener,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let input = register_global_input_listener()?;
        Ok(Self { input })
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        while let Some(event) = self.input.poll() {
            println!("got input: {event:#?}");
        }
        Ok(())
    }
}
