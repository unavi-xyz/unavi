use std::cell::Cell;

use crate::wired::event::{
    api::{emit, listen},
    types::{EventFilter, EventReceptor, EventScope},
};

wired_prelude::generate_script!(Script);

struct Script {
    receptor: EventReceptor,
    tick_n: Cell<u32>,
}

impl GuestScript for Script {
    fn new() -> Self {
        let receptor = listen(
            &["test:channel".to_string()],
            EventFilter {
                node: None,
                scope: EventScope::Global,
                documents: None,
            },
        );
        Self {
            receptor,
            tick_n: Cell::new(0),
        }
    }

    fn tick(&self) {
        match self.tick_n.get() {
            0 => {
                emit(
                    "test:channel",
                    &[1_u8, 2, 3],
                    EventFilter {
                        node: None,
                        scope: EventScope::Global,
                        documents: None,
                    },
                );
            }
            1 => {
                match self.receptor.poll() {
                    Some(ev) => {
                        if ev.channel == "test:channel" {
                            println!("pass: event channel");
                        } else {
                            eprintln!("FAIL event channel: got {:?}", ev.channel);
                        }
                        if ev.payload == [1_u8, 2, 3] {
                            println!("pass: event payload");
                        } else {
                            eprintln!("FAIL event payload: got {:?}", ev.payload);
                        }
                    }
                    None => eprintln!("FAIL event: no event received after emit"),
                }
                println!("tests complete");
            }
            _ => {}
        }
        self.tick_n.set(self.tick_n.get() + 1);
    }

    fn render(&self) {}

    fn drop(&self) {}
}
