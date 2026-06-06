use crate::{
    exports::unavi::vui_module::api::{
        GuestVuiModule,
        ModuleEvent,
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

pub struct VuiModule {
    name:              String,
    icon_prim_id:      String,
    request_receptor:  EventReceptor,
    activate_receptor: EventReceptor,
}

impl GuestVuiModule for VuiModule {
    fn new(name: String, icon: &Prim) -> Self {
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
                CH_SET_COLOR.to_string(),
            ],
            EventFilter {
                documents: None,
                scope:     EventScope::Global,
            },
        )
        .expect("listen");
        Self {
            name,
            icon_prim_id,
            request_receptor,
            activate_receptor,
        }
    }

    fn poll(&self) -> Option<ModuleEvent> {
        while let Some(event) = self.request_receptor.poll() {
            let payload = postcard::to_allocvec(&RegisterPayload {
                name:         self.name.clone(),
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
                        return Some(ModuleEvent::Activate(p.transform));
                    }
                }
                CH_DEACTIVATE => return Some(ModuleEvent::Deactivate),
                CH_SET_COLOR => {
                    if let Ok(p) = postcard::from_bytes::<SetColorPayload>(&event.payload()) {
                        return Some(ModuleEvent::SetColor(p.color));
                    }
                }
                _ => {}
            }
        }

        None
    }
}
