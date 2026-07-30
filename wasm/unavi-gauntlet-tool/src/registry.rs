use std::cell::Cell;

use wired_prelude::{
    wired_math::types::Transform,
    wired_scene::types::Color,
};

use crate::{
    exports::unavi::gauntlet_tool::api::{
        GuestToolRegistry,
        RegisteredTool,
        ToolState,
    },
    protocol::{
        ActivatePayload,
        CH_ACTIVATE,
        CH_DEACTIVATE,
        CH_DISCOVER,
        CH_REGISTER,
        CH_SCROLL,
        CH_SET_STATE,
        CH_TRIGGER,
        RegisterPayload,
        ScrollPayload,
        ToolStatePayload,
        TriggerPayload,
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

pub struct ToolRegistry {
    register_receptor: EventReceptor,
    ticks:             Cell<u32>,
    fired:             Cell<bool>,
}

impl GuestToolRegistry for ToolRegistry {
    fn new() -> Self {
        let register_receptor = listen(
            &[CH_REGISTER.to_string()],
            EventFilter {
                documents: None,
                scope:     EventScope::Global,
            },
        )
        .expect("listen");
        Self {
            register_receptor,
            ticks: Cell::new(0),
            fired: Cell::new(false),
        }
    }

    fn poll(&self) -> Vec<RegisteredTool> {
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
                )
                .ok();
                self.fired.set(true);
            }
        }

        let mut results = Vec::new();
        while let Some(event) = self.register_receptor.poll() {
            if let Ok(p) = postcard::from_bytes::<RegisterPayload>(&event.payload()) {
                results.push(RegisteredTool {
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
        )
        .ok();
    }

    fn deactivate(&self, doc_id: Vec<u8>) {
        emit(
            CH_DEACTIVATE,
            &[],
            EventFilter {
                documents: Some(vec![doc_id]),
                scope:     EventScope::Global,
            },
        )
        .ok();
    }

    fn set_state(&self, doc_id: Vec<u8>, state: ToolState) {
        let payload = postcard::to_allocvec(&ToolStatePayload {
            color:  state.color,
            in_use: state.in_use,
        })
        .expect("encode set state");
        emit(
            CH_SET_STATE,
            &payload,
            EventFilter {
                documents: Some(vec![doc_id]),
                scope:     EventScope::Global,
            },
        )
        .ok();
    }

    fn trigger(&self, doc_id: Vec<u8>, pressed: bool) {
        let payload = postcard::to_allocvec(&TriggerPayload { pressed }).expect("encode trigger");
        emit(
            CH_TRIGGER,
            &payload,
            EventFilter {
                documents: Some(vec![doc_id]),
                scope:     EventScope::Global,
            },
        )
        .ok();
    }

    fn scroll(&self, doc_id: Vec<u8>, delta: f32) {
        let payload = postcard::to_allocvec(&ScrollPayload { delta }).expect("encode scroll");
        emit(
            CH_SCROLL,
            &payload,
            EventFilter {
                documents: Some(vec![doc_id]),
                scope:     EventScope::Global,
            },
        )
        .ok();
    }
}

impl From<Color> for ToolState {
    fn from(color: Color) -> Self {
        Self {
            color,
            in_use: false,
        }
    }
}
