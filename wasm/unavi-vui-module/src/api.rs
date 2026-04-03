use crate::wired::scene::types::Node;
use crate::{
    exports::unavi::vui_module::api::{ActivateTransform, GuestVuiModule, ModuleEvent},
    protocol::{
        ActivatePayload, CH_ACTIVATE, CH_DEACTIVATE, CH_DISCOVER, CH_REGISTER, RegisterPayload,
    },
    wired::event::{
        api::{register_emitter, register_receptor},
        types::EventReceptor,
    },
};
use wired_prelude::wired_math::types::{Quat, Vec3};
use wired_prelude::wired_scene::types::Color;

pub struct VuiModuleImpl {
    name: String,
    color: [f32; 4],
    icon_node_id: String,
    request_receptor: EventReceptor,
    activate_receptor: EventReceptor,
}

impl GuestVuiModule for VuiModuleImpl {
    fn new(name: String, color: Color, icon: &Node) -> Self {
        let icon_node_id = icon.id();
        let request_receptor = register_receptor(&[CH_DISCOVER.to_string()], None, f32::MAX, &[]);
        let activate_receptor = register_receptor(
            &[CH_ACTIVATE.to_string(), CH_DEACTIVATE.to_string()],
            None,
            f32::MAX,
            &[],
        );
        Self {
            name,
            color: [color.r, color.g, color.b, color.a],
            icon_node_id,
            request_receptor,
            activate_receptor,
        }
    }

    fn poll(&self) -> Option<ModuleEvent> {
        while let Some(event) = self.request_receptor.poll() {
            let payload = postcard::to_allocvec(&RegisterPayload {
                name: self.name.clone(),
                color: self.color,
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
                        return Some(ModuleEvent::Activate(ActivateTransform {
                            translation: Vec3 {
                                x: p.translation[0],
                                y: p.translation[1],
                                z: p.translation[2],
                            },
                            rotation: Quat {
                                x: p.rotation[0],
                                y: p.rotation[1],
                                z: p.rotation[2],
                                w: p.rotation[3],
                            },
                            scale: Vec3 {
                                x: p.scale[0],
                                y: p.scale[1],
                                z: p.scale[2],
                            },
                        }));
                    }
                }
                CH_DEACTIVATE => return Some(ModuleEvent::Deactivate),
                _ => {}
            }
        }

        None
    }
}
