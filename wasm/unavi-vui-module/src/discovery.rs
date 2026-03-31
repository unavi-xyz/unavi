use crate::{
    exports::unavi::vui_module::discovery::{GuestModuleDiscovery, RegisteredModule},
    protocol::{
        ActivatePayload, CH_ACTIVATE, CH_DEACTIVATE, CH_DISCOVER, CH_REGISTER, RegisterPayload,
    },
    wired::event::{
        api::{register_emitter, register_receptor},
        types::{EventEmitter, EventReceptor},
    },
};
use wired_prelude::wired_scene::types::Color;

pub struct ModuleDiscoveryImpl {
    _emitter: EventEmitter,
    register_receptor: EventReceptor,
}

impl GuestModuleDiscovery for ModuleDiscoveryImpl {
    fn new() -> Self {
        let emitter = register_emitter(None, f32::MAX, &[]);
        emitter.emit(CH_DISCOVER, &[]);
        let register_receptor = register_receptor(&[CH_REGISTER.to_string()], None, f32::MAX, &[]);
        Self {
            _emitter: emitter,
            register_receptor,
        }
    }

    fn poll(&self) -> Vec<RegisteredModule> {
        let mut results = Vec::new();
        while let Some(event) = self.register_receptor.poll() {
            if let Ok(p) = postcard::from_bytes::<RegisterPayload>(&event.payload) {
                results.push(RegisteredModule {
                    doc_id: event.sender_document,
                    name: p.name,
                    color: Color {
                        r: p.color[0],
                        g: p.color[1],
                        b: p.color[2],
                        a: p.color[3],
                    },
                });
            }
        }
        results
    }

    fn activate(
        &self,
        doc_id: Vec<u8>,
        translation: wired_prelude::wired_math::types::Vec3,
        rotation: wired_prelude::wired_math::types::Quat,
        scale: wired_prelude::wired_math::types::Vec3,
    ) {
        let payload = postcard::to_allocvec(&ActivatePayload {
            translation: [translation.x, translation.y, translation.z],
            rotation: [rotation.x, rotation.y, rotation.z, rotation.w],
            scale: [scale.x, scale.y, scale.z],
        })
        .expect("encode activate");
        let emitter = register_emitter(None, f32::MAX, &[doc_id]);
        emitter.emit(CH_ACTIVATE, &payload);
    }

    fn deactivate(&self, doc_id: Vec<u8>) {
        let emitter = register_emitter(None, f32::MAX, &[doc_id]);
        emitter.emit(CH_DEACTIVATE, &[]);
    }
}
