use std::cell::Cell;

use crate::{
    exports::unavi::vui_module::api::{GuestVuiModuleRegistry, RegisteredModule},
    protocol::{
        ActivatePayload, CH_ACTIVATE, CH_DEACTIVATE, CH_DISCOVER, CH_REGISTER, CH_SET_COLOR,
        RegisterPayload, SetColorPayload,
    },
    wired::event::{
        api::{register_emitter, register_receptor},
        types::{EventEmitter, EventReceptor},
    },
};
use wired_prelude::{wired_math::types::Transform, wired_scene::types::Color};

// TODO: re-discover on an interval, or some other non-time-based method
/// Discovery delay, to let other scripts load.
const DISCOVER_DELAY_TICKS: u32 = 60;

pub struct VuiModuleRegistry {
    emitter: EventEmitter,
    register_receptor: EventReceptor,
    ticks: Cell<u32>,
    fired: Cell<bool>,
}

impl GuestVuiModuleRegistry for VuiModuleRegistry {
    fn new() -> Self {
        let emitter = register_emitter(None, f32::MAX, &[]);
        let register_receptor = register_receptor(&[CH_REGISTER.to_string()], None, f32::MAX, &[]);
        Self {
            emitter,
            register_receptor,
            ticks: Cell::new(0),
            fired: Cell::new(false),
        }
    }

    fn poll(&self) -> Vec<RegisteredModule> {
        if !self.fired.get() {
            let t = self.ticks.get() + 1;
            self.ticks.set(t);

            if t >= DISCOVER_DELAY_TICKS {
                self.emitter.emit(CH_DISCOVER, &[]);
                self.fired.set(true);
            }
        }

        let mut results = Vec::new();
        while let Some(event) = self.register_receptor.poll() {
            if let Ok(p) = postcard::from_bytes::<RegisterPayload>(&event.payload) {
                println!("found module: {}", p.name);
                results.push(RegisteredModule {
                    doc_id: event.sender_document,
                    name: p.name,
                    icon_node_id: p.icon_node_id,
                });
            } else {
                eprintln!("received invalid event payload");
            }
        }
        results
    }

    fn activate(&self, doc_id: Vec<u8>, transform: Transform) {
        let payload =
            postcard::to_allocvec(&ActivatePayload { transform }).expect("encode activate");
        let emitter = register_emitter(None, f32::MAX, &[doc_id]);
        emitter.emit(CH_ACTIVATE, &payload);
    }

    fn deactivate(&self, doc_id: Vec<u8>) {
        let emitter = register_emitter(None, f32::MAX, &[doc_id]);
        emitter.emit(CH_DEACTIVATE, &[]);
    }

    fn set_color(&self, doc_id: Vec<u8>, color: Color) {
        let payload = postcard::to_allocvec(&SetColorPayload { color }).expect("encode set color");
        let emitter = register_emitter(None, f32::MAX, &[doc_id]);
        emitter.emit(CH_SET_COLOR, &payload);
    }
}
