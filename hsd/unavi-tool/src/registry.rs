//! The shell side of `unavi:tool/api`: the belt a halo-like shell polls for
//! announced tools and pushes state onto.

use std::{
    cell::{
        Cell,
        RefCell,
    },
    collections::HashSet,
};

use crate::{
    exports::unavi::tool::api::{
        Glyph,
        GuestToolRegistry,
        RegisteredTool,
        ToolButton,
        ToolState,
    },
    protocol::{
        self,
        ActivatePayload,
        CH_ACTIVATE,
        CH_DEACTIVATE,
        CH_DISCOVER,
        CH_HOLD_GRIP,
        CH_PRESS,
        CH_REGISTER,
        CH_SCROLL,
        CH_SET_STATE,
        HoldGripPayload,
        MAX_DESCRIPTION_BYTES,
        MAX_NAME_BYTES,
        PressPayload,
        RegisterPayload,
        ScrollPayload,
        ToolStatePayload,
    },
    wired::{
        event::messaging::{
            self,
            MessageSubscription,
            Scope,
        },
        input::types::PointerKind,
    },
};

const DRAIN_MAX: u32 = 16;

/// How often the belt asks every loaded tool to announce itself. Repeating
/// rather than firing once means a `tool` that finishes loading after an
/// earlier announcement is still found, within one interval.
const DISCOVER_INTERVAL_TICKS: u32 = 60;

pub struct ToolRegistry {
    /// `None` when opening the listener failed; the belt then never
    /// discovers a tool, instead of trapping the whole script.
    register_rx: Option<MessageSubscription>,
    grip_rx:     Option<MessageSubscription>,
    /// The tools in hand, whose grip holds are the only ones taken.
    active:      RefCell<HashSet<(u64, u64, u64, u64)>>,
    /// The tools that hold their hand's grip.
    gripping:    RefCell<HashSet<(u64, u64, u64, u64)>>,
    ticks:       Cell<u32>,
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
        let grip_rx = match messaging::listen(&[CH_HOLD_GRIP.to_owned()], None, Scope::Global) {
            Ok(rx) => Some(rx),
            Err(err) => {
                eprintln!("tool-registry: listen: {err:?}");
                None
            }
        };
        Self {
            register_rx,
            grip_rx,
            active: RefCell::new(HashSet::new()),
            gripping: RefCell::new(HashSet::new()),
            ticks: Cell::new(0),
        }
    }

    fn poll(&self) -> Vec<RegisteredTool> {
        let ticks = self.ticks.get() + 1;
        self.ticks.set(ticks);
        if ticks.is_multiple_of(DISCOVER_INTERVAL_TICKS)
            && let Err(err) = messaging::emit(CH_DISCOVER, &[], None, Scope::Global)
        {
            eprintln!("tool-registry: emit discover: {err:?}");
        }

        self.read_grips();
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
                // A glyph that breaks the rules is dropped rather than the
                // tool: it is still a tool, just one drawn with the generic
                // mark.
                let glyph = p.glyph.and_then(protocol::accept_glyph).map(|glyph| Glyph {
                    size: glyph.size,
                    bits: glyph.bits,
                });
                Some(RegisteredTool {
                    document,
                    name: p.name,
                    description: p.description,
                    glyph,
                })
            })
            .collect()
    }

    fn activate(&self, document: (u64, u64, u64, u64), hand: PointerKind) {
        self.active.borrow_mut().insert(document);
        emit_to(
            document,
            CH_ACTIVATE,
            &ActivatePayload { hand: hand.into() },
        );
    }

    fn holds_grip(&self, document: (u64, u64, u64, u64)) -> bool {
        self.gripping.borrow().contains(&document)
    }

    fn deactivate(&self, document: (u64, u64, u64, u64)) {
        // A tool put away lets go of everything, the grip with it, and a hold
        // it sent on its way out is not taken.
        self.active.borrow_mut().remove(&document);
        self.gripping.borrow_mut().remove(&document);
        if let Err(err) = messaging::emit(CH_DEACTIVATE, &[], Some(&[document]), Scope::Global) {
            eprintln!("tool-registry: emit deactivate: {err:?}");
        }
    }

    fn set_state(&self, document: (u64, u64, u64, u64), state: ToolState) {
        emit_to(
            document,
            CH_SET_STATE,
            &ToolStatePayload { color: state.color },
        );
    }

    fn press(&self, document: (u64, u64, u64, u64), button: ToolButton, pressed: bool) {
        emit_to(
            document,
            CH_PRESS,
            &PressPayload {
                button: button.into(),
                pressed,
            },
        );
    }

    fn scroll(&self, document: (u64, u64, u64, u64), delta: f32) {
        emit_to(document, CH_SCROLL, &ScrollPayload { delta });
    }
}

impl ToolRegistry {
    fn read_grips(&self) {
        let Some(grip_rx) = &self.grip_rx else {
            return;
        };
        let active = self.active.borrow();
        let mut gripping = self.gripping.borrow_mut();
        for message in grip_rx.drain(DRAIN_MAX) {
            let (Some(document), Ok(payload)) = (
                message.sender,
                postcard::from_bytes::<HoldGripPayload>(&message.payload),
            ) else {
                continue;
            };
            if payload.held && active.contains(&document) {
                gripping.insert(document);
            } else {
                gripping.remove(&document);
            }
        }
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
