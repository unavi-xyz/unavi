use crate::protocol::{CH_SET_COLOR, SetColorPayload};
use crate::wired::scene::types::Mesh;
use crate::{
    exports::unavi::vui_module::api::{GuestVuiModule, ModuleEvent},
    protocol::{
        ActivatePayload, CH_ACTIVATE, CH_DEACTIVATE, CH_DISCOVER, CH_REGISTER, RegisterPayload,
    },
    wired::event::{
        api::{emit, listen},
        types::{EventFilter, EventReceptor, EventScope},
    },
};

pub struct VuiModule {
    name: String,
    icon_mesh_id: String,
    request_receptor: EventReceptor,
    activate_receptor: EventReceptor,
}

impl GuestVuiModule for VuiModule {
    fn new(name: String, icon: &Mesh) -> Self {
        let icon_mesh_id = icon.id();
        let request_receptor = listen(
            &[CH_DISCOVER.to_string()],
            EventFilter {
                node: None,
                scope: EventScope::Global,
                documents: None,
            },
        );
        let activate_receptor = listen(
            &[
                CH_ACTIVATE.to_string(),
                CH_DEACTIVATE.to_string(),
                CH_SET_COLOR.to_string(),
            ],
            EventFilter {
                node: None,
                scope: EventScope::Global,
                documents: None,
            },
        );
        Self {
            name,
            icon_mesh_id,
            request_receptor,
            activate_receptor,
        }
    }

    fn poll(&self) -> Option<ModuleEvent> {
        while let Some(event) = self.request_receptor.poll() {
            let payload = postcard::to_allocvec(&RegisterPayload {
                name: self.name.clone(),
                icon_mesh_id: self.icon_mesh_id.clone(),
            })
            .expect("encode register");
            emit(
                CH_REGISTER,
                &payload,
                EventFilter {
                    node: None,
                    scope: EventScope::Global,
                    documents: Some(vec![event.sender_document]),
                },
            );
        }

        while let Some(event) = self.activate_receptor.poll() {
            match event.channel.as_str() {
                CH_ACTIVATE => {
                    if let Ok(p) = postcard::from_bytes::<ActivatePayload>(&event.payload) {
                        return Some(ModuleEvent::Activate(p.transform));
                    }
                }
                CH_DEACTIVATE => return Some(ModuleEvent::Deactivate),
                CH_SET_COLOR => {
                    if let Ok(p) = postcard::from_bytes::<SetColorPayload>(&event.payload) {
                        return Some(ModuleEvent::SetColor(p.color));
                    }
                }
                _ => {}
            }
        }

        None
    }
}
