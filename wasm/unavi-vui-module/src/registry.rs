use std::cell::Cell;

use wired_prelude::{
    wired_math::types::Transform,
    wired_scene::types::Color,
};

use crate::{
    exports::unavi::vui_module::api::{
        GuestVuiModuleRegistry,
        RegisteredModule,
    },
    protocol::{
        ActivatePayload,
        CH_ACTIVATE,
        CH_DEACTIVATE,
        CH_DISCOVER,
        CH_REGISTER,
        CH_SET_COLOR,
        RegisterPayload,
        SetColorPayload,
    },
    wired::event::{
        api::{
            emit,
            listen,
        },
        types::{
            EventFilter,
            EventReceptor,
            EventScope,
        },
    },
};

// TODO: re-discover on an interval, or some other non-time-based method
/// Discovery delay, to let other scripts load.
const DISCOVER_DELAY_TICKS: u32 = 60;

pub struct VuiModuleRegistry {
    register_receptor: EventReceptor,
    ticks:             Cell<u32>,
    fired:             Cell<bool>,
}

impl GuestVuiModuleRegistry for VuiModuleRegistry {
    fn new() -> Self {
        let register_receptor = listen(
            &[CH_REGISTER.to_string()],
            EventFilter {
                documents: None,
                scope:     EventScope::Global,
            },
        );
        Self {
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
                emit(
                    CH_DISCOVER,
                    &[],
                    EventFilter {
                        documents: None,
                        scope:     EventScope::Global,
                    },
                );
                self.fired.set(true);
            }
        }

        let mut results = Vec::new();
        while let Some(event) = self.register_receptor.poll() {
            if let Ok(p) = postcard::from_bytes::<RegisterPayload>(&event.payload()) {
                results.push(RegisteredModule {
                    doc_id:       event.sender().document,
                    name:         p.name,
                    icon_prim_id: p.icon_prim_id,
                });
            } else {
                eprintln!("Received invalid event payload");
            }
        }
        results
    }

    fn activate(&self, doc_id: Vec<u8>, transform: Transform) {
        let payload =
            postcard::to_allocvec(&ActivatePayload { transform }).expect("encode activate");
        emit(
            CH_ACTIVATE,
            &payload,
            EventFilter {
                documents: Some(vec![doc_id]),
                scope:     EventScope::Global,
            },
        );
    }

    fn deactivate(&self, doc_id: Vec<u8>) {
        emit(
            CH_DEACTIVATE,
            &[],
            EventFilter {
                documents: Some(vec![doc_id]),
                scope:     EventScope::Global,
            },
        );
    }

    fn set_color(&self, doc_id: Vec<u8>, color: Color) {
        let payload = postcard::to_allocvec(&SetColorPayload { color }).expect("encode set color");
        emit(
            CH_SET_COLOR,
            &payload,
            EventFilter {
                documents: Some(vec![doc_id]),
                scope:     EventScope::Global,
            },
        );
    }
}
