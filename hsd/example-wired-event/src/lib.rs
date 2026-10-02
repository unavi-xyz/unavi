//! Emits and listens for a message on its own global channel, to show
//! `wired:event/messaging` round-tripping to itself.

use crate::wired::event::messaging::{
    MessageSubscription,
    Scope,
};

wired_guest::generate_script!(Script);

const CHANNEL: &str = "example:wired-event/my-event";

struct Script {
    subscription: MessageSubscription,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let subscription =
            wired::event::messaging::listen(&[CHANNEL.to_owned()], None, Scope::Global)?;

        wired::event::messaging::emit(CHANNEL, b"hello, world!", None, Scope::Global)?;

        Ok(Self { subscription })
    }

    fn fixed_update(
        &mut self,
        _tick: exports::wired::script::lifecycle::Tick,
    ) -> anyhow::Result<()> {
        for message in self.subscription.drain(32) {
            println!(
                "-> Got message on {}: {:?}",
                message.channel, message.payload
            );
        }
        Ok(())
    }
}
