use crate::{
    exports::unavi::tool::api::{
        GuestTool,
        ToolEvent,
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
    wired::{
        event::{
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
        scene::types::Prim,
    },
};

pub struct Tool {
    name:              String,
    description:       String,
    icon_prim_id:      String,
    request_receptor:  EventReceptor,
    activate_receptor: EventReceptor,
}

impl GuestTool for Tool {
    fn new(name: String, description: String, icon: &Prim) -> Self {
        let icon_prim_id = icon.id();
        let request_receptor = listen(
            &[CH_DISCOVER.to_string()],
            EventFilter {
                documents: None,
                scope:     EventScope::Global,
            },
        )
        .expect("listen");
        let activate_receptor = listen(
            &[
                CH_ACTIVATE.to_string(),
                CH_DEACTIVATE.to_string(),
                CH_SET_STATE.to_string(),
                CH_TRIGGER.to_string(),
                CH_SCROLL.to_string(),
            ],
            EventFilter {
                documents: None,
                scope:     EventScope::Global,
            },
        )
        .expect("listen");
        Self {
            name,
            description,
            icon_prim_id,
            request_receptor,
            activate_receptor,
        }
    }

    fn poll(&self) -> Option<ToolEvent> {
        while let Some(event) = self.request_receptor.poll() {
            let payload = postcard::to_allocvec(&RegisterPayload {
                name:         self.name.clone(),
                description:  self.description.clone(),
                icon_prim_id: self.icon_prim_id.clone(),
            })
            .expect("encode register");
            emit(
                CH_REGISTER,
                &payload,
                EventFilter {
                    documents: Some(vec![event.sender().document]),
                    scope:     EventScope::Global,
                },
            )
            .ok();
        }

        while let Some(event) = self.activate_receptor.poll() {
            match event.channel().as_str() {
                CH_ACTIVATE => {
                    if let Ok(p) = postcard::from_bytes::<ActivatePayload>(&event.payload()) {
                        return Some(ToolEvent::Activate(p.transform));
                    }
                }
                CH_DEACTIVATE => return Some(ToolEvent::Deactivate),
                CH_SET_STATE => {
                    if let Ok(p) = postcard::from_bytes::<ToolStatePayload>(&event.payload()) {
                        return Some(ToolEvent::SetState(ToolState {
                            color:  p.color,
                            in_use: p.in_use,
                        }));
                    }
                }
                CH_TRIGGER => {
                    if let Ok(p) = postcard::from_bytes::<TriggerPayload>(&event.payload()) {
                        return Some(ToolEvent::Trigger(p.pressed));
                    }
                }
                CH_SCROLL => {
                    if let Ok(p) = postcard::from_bytes::<ScrollPayload>(&event.payload()) {
                        return Some(ToolEvent::Scroll(p.delta));
                    }
                }
                _ => {}
            }
        }

        None
    }
}
