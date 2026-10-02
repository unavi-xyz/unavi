use crate::wired::input::types::InputSubscription;

wired_guest::generate_script!(Script);

struct Script {
    input: InputSubscription,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let input = wired::input::device::listen()?;
        Ok(Self { input })
    }

    fn fixed_update(
        &mut self,
        _tick: exports::wired::script::lifecycle::Tick,
    ) -> anyhow::Result<()> {
        for event in self.input.drain(32) {
            println!("got input: {event:#?}");
        }
        Ok(())
    }
}
