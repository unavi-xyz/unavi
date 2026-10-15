//! The tool side of `unavi:tool/api`: a tool crate builds one `tool` and
//! polls it for what the shell asks of it.

use std::{
    cell::RefCell,
    collections::VecDeque,
};

use crate::{
    exports::unavi::tool::api::{
        GuestTool,
        ToolEvent,
        ToolState,
    },
    protocol::{
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
    wired::{
        core::error::Error,
        event::messaging::{
            self,
            MessageSubscription,
            Scope,
        },
    },
};

/// Messages drained from the network each `poll`, queued so a tick with
/// several arrivals does not drop any.
const DRAIN_MAX: u32 = 16;

pub struct Tool {
    name:        String,
    description: String,
    discover_rx: MessageSubscription,
    control_rx:  MessageSubscription,
    pending:     RefCell<VecDeque<ToolEvent>>,
}

impl GuestTool for Tool {
    fn new(name: String, description: String) -> Result<Self, Error> {
        if name.len() > MAX_NAME_BYTES {
            return Err(Error::InvalidArgument(format!(
                "name must be at most {MAX_NAME_BYTES} bytes"
            )));
        }
        if description.len() > MAX_DESCRIPTION_BYTES {
            return Err(Error::InvalidArgument(format!(
                "description must be at most {MAX_DESCRIPTION_BYTES} bytes"
            )));
        }

        let discover_rx = messaging::listen(&[CH_DISCOVER.to_owned()], None, Scope::Global)?;
        let control_rx = messaging::listen(
            &[
                CH_ACTIVATE.to_owned(),
                CH_DEACTIVATE.to_owned(),
                CH_SET_STATE.to_owned(),
                CH_TRIGGER.to_owned(),
                CH_SCROLL.to_owned(),
            ],
            None,
            Scope::Global,
        )?;

        Ok(Self {
            name,
            description,
            discover_rx,
            control_rx,
            pending: RefCell::new(VecDeque::new()),
        })
    }

    fn poll(&self) -> Option<ToolEvent> {
        for message in self.discover_rx.drain(DRAIN_MAX) {
            let Some(sender) = message.sender else {
                continue;
            };
            let payload = match postcard::to_allocvec(&RegisterPayload {
                name:        self.name.clone(),
                description: self.description.clone(),
            }) {
                Ok(payload) => payload,
                Err(err) => {
                    eprintln!("tool: encode register: {err}");
                    continue;
                }
            };
            if let Err(err) = messaging::emit(CH_REGISTER, &payload, Some(&[sender]), Scope::Global)
            {
                eprintln!("tool: emit register: {err:?}");
            }
        }

        let mut pending = self.pending.borrow_mut();
        for message in self.control_rx.drain(DRAIN_MAX) {
            let event = match message.channel.as_str() {
                CH_ACTIVATE => Some(ToolEvent::Activate),
                CH_DEACTIVATE => Some(ToolEvent::Deactivate),
                CH_SET_STATE => postcard::from_bytes::<ToolStatePayload>(&message.payload)
                    .ok()
                    .map(|p| ToolEvent::SetState(ToolState { color: p.color })),
                CH_TRIGGER => postcard::from_bytes::<TriggerPayload>(&message.payload)
                    .ok()
                    .map(|p| ToolEvent::Trigger(p.pressed)),
                CH_SCROLL => postcard::from_bytes::<ScrollPayload>(&message.payload)
                    .ok()
                    .map(|p| ToolEvent::Scroll(p.delta)),
                _ => None,
            };
            match event {
                Some(event) => pending.push_back(event),
                None => eprintln!("tool: malformed message on {}", message.channel),
            }
        }

        pending.pop_front()
    }
}
