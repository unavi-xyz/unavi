use crate::wired::event::types::{
    EventFilter,
    EventReceptor,
    EventScope,
};

wired_prelude::generate_script!(Script);

const CHANNEL: &str = "my-event";

struct Script {
    receptor: EventReceptor,
}

impl ScriptBehavior for Script {
    fn init() -> Self {
        let receptor = wired::event::api::listen(
            &[CHANNEL.to_string()],
            EventFilter {
                documents: None,
                scope:     EventScope::Global,
            },
        );

        wired::event::api::emit(
            CHANNEL,
            b"hello, world!",
            EventFilter {
                documents: None,
                scope:     EventScope::Global,
            },
        );

        Self { receptor }
    }

    fn tick(&mut self) {
        while let Some(event) = self.receptor.poll() {
            println!("-> Got event on {}: {:?}", event.channel(), event.payload());
        }
    }
}
