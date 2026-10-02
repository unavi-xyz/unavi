//! The shell side of `unavi:tool/api`: the belt a halo-like shell polls for
//! announced tools and pushes state onto.

use std::cell::Cell;

use wired_guest::math::Transform;

use crate::{
    exports::unavi::tool::api::{
        GuestToolRegistry,
        RegisteredTool,
        ToolState,
    },
    protocol::{
        self,
        ActivatePayload,
        CH_ACTIVATE,
        CH_DEACTIVATE,
        CH_DISCOVER,
        CH_REGISTER,
        CH_SCROLL,
        CH_SET_STATE,
        CH_TRIGGER,
        MAX_DESCRIPTION_BYTES,
        MAX_NAME_BYTES,
        RegisterPayload,
        ScrollPayload,
        ToolStatePayload,
        TriggerPayload,
    },
    wired::event::messaging::{
        self,
        MessageSubscription,
        Scope,
    },
};

const DRAIN_MAX: u32 = 16;

/// Ticks before the belt asks every loaded tool to announce itself.
/// `tool`s loaded after this fire are never discovered; see the WIT pain
/// points in the step 5c handoff.
const DISCOVER_DELAY_TICKS: u32 = 60;

pub struct ToolRegistry {
    /// `None` when opening the listener failed; the belt then never
    /// discovers a tool, instead of trapping the whole script.
    register_rx: Option<MessageSubscription>,
    ticks:       Cell<u32>,
    discovered:  Cell<bool>,
}

impl GuestToolRegistry for ToolRegistry {
    fn new() -> Self {
        let register_rx = match messaging::listen(&[CH_REGISTER.to_owned()], None, Scope::Global) {
            Ok(rx) => Some(rx),
            Err(err) => {
                eprintln!("tool-registry: listen: {err:?}");
                None
            }
        };
        Self {
            register_rx,
            ticks: Cell::new(0),
            discovered: Cell::new(false),
        }
    }

    fn poll(&self) -> Vec<RegisteredTool> {
        if !self.discovered.get() {
            let ticks = self.ticks.get() + 1;
            self.ticks.set(ticks);
            if ticks >= DISCOVER_DELAY_TICKS {
                self.discovered.set(true);
                if let Err(err) = messaging::emit(CH_DISCOVER, &[], None, Scope::Global) {
                    eprintln!("tool-registry: emit discover: {err:?}");
                }
            }
        }

        let Some(register_rx) = &self.register_rx else {
            return Vec::new();
        };

        register_rx
            .drain(DRAIN_MAX)
            .into_iter()
            .filter_map(|message| {
                let document = message.sender?;
                let mut p = postcard::from_bytes::<RegisterPayload>(&message.payload).ok()?;
                protocol::truncate(&mut p.name, MAX_NAME_BYTES);
                protocol::truncate(&mut p.description, MAX_DESCRIPTION_BYTES);
                Some(RegisteredTool {
                    document,
                    name: p.name,
                    description: p.description,
                })
            })
            .collect()
    }

    fn activate(&self, document: (u64, u64, u64, u64), transform: Transform) {
        emit_to(document, CH_ACTIVATE, &ActivatePayload { transform });
    }

    fn deactivate(&self, document: (u64, u64, u64, u64)) {
        if let Err(err) = messaging::emit(CH_DEACTIVATE, &[], Some(&[document]), Scope::Global) {
            eprintln!("tool-registry: emit deactivate: {err:?}");
        }
    }

    fn set_state(&self, document: (u64, u64, u64, u64), state: ToolState) {
        emit_to(
            document,
            CH_SET_STATE,
            &ToolStatePayload {
                color:  state.color,
                in_use: state.in_use,
            },
        );
    }

    fn trigger(&self, document: (u64, u64, u64, u64), pressed: bool) {
        emit_to(document, CH_TRIGGER, &TriggerPayload { pressed });
    }

    fn scroll(&self, document: (u64, u64, u64, u64), delta: f32) {
        emit_to(document, CH_SCROLL, &ScrollPayload { delta });
    }
}

fn emit_to(document: (u64, u64, u64, u64), channel: &str, payload: &impl serde::Serialize) {
    let payload = match postcard::to_allocvec(payload) {
        Ok(payload) => payload,
        Err(err) => {
            eprintln!("tool-registry: encode {channel}: {err}");
            return;
        }
    };
    if let Err(err) = messaging::emit(channel, &payload, Some(&[document]), Scope::Global) {
        eprintln!("tool-registry: emit {channel}: {err:?}");
    }
}
