use std::cell::Cell;

use crate::wired::event::{
    api::{register_emitter, register_receptor},
    types::{EventEmitter, EventReceptor},
};

wired_prelude::generate_script!(Script);

struct Script {
    emitter:  EventEmitter,
    receptor: EventReceptor,
    tick_n:   Cell<u32>,
}

impl GuestScript for Script {
    fn new() -> Self {
        let channels = ["test:channel".to_string()];
        let emitter  = register_emitter(None, 0.0, &[]);
        let receptor = register_receptor(&channels, None, 0.0, &[]);
        Self { emitter, receptor, tick_n: Cell::new(0) }
    }

    fn tick(&self) {
        match self.tick_n.get() {
            0 => {
                self.emitter.emit("test:channel", &[1_u8, 2, 3]);
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
