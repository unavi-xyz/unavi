use crate::protocol::{CH_SET_COLOR, SetColorPayload};
use crate::wired::scene::types::Node;
use crate::{
    exports::unavi::vui_module::api::{GuestVuiModule, ModuleEvent},
    protocol::{
        ActivatePayload, CH_ACTIVATE, CH_DEACTIVATE, CH_DISCOVER, CH_REGISTER, RegisterPayload,
    },
    wired::event::{
        api::{register_emitter, register_receptor},
        types::EventReceptor,
    },
};

pub struct VuiModule {
    name: String,
    icon_node_id: String,
    request_receptor: EventReceptor,
    activate_receptor: EventReceptor,
}

impl GuestVuiModule for VuiModule {
    fn new(name: String, icon: &Node) -> Self {
        let icon_node_id = icon.id();
        let request_receptor = register_receptor(&[CH_DISCOVER.to_string()], None, f32::MAX, &[]);
        let activate_receptor = register_receptor(
            &[
                CH_ACTIVATE.to_string(),
                CH_DEACTIVATE.to_string(),
                CH_SET_COLOR.to_string(),
            ],
            None,
            f32::MAX,
            &[],
        );
        Self {
            name,
            icon_node_id,
            request_receptor,
            activate_receptor,
        }
    }

    fn poll(&self) -> Option<ModuleEvent> {
        while let Some(event) = self.request_receptor.poll() {
            let payload = postcard::to_allocvec(&RegisterPayload {
                name: self.name.clone(),
                icon_node_id: self.icon_node_id.clone(),
            })
            .expect("encode register");
            let emitter = register_emitter(None, f32::MAX, &[event.sender_document]);
            emitter.emit(CH_REGISTER, &payload);
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
