use std::cell::Cell;

use crate::{
    exports::unavi::vui_module::discovery::{GuestModuleDiscovery, RegisteredModule},
    protocol::{ActivatePayload, CH_ACTIVATE, CH_DEACTIVATE, CH_DISCOVER, CH_REGISTER, RegisterPayload},
    wired::event::{
        api::{register_emitter, register_receptor},
        types::{EventEmitter, EventReceptor},
    },
};
use wired_prelude::wired_scene::types::Color;

/// Ticks to wait before emitting CH_DISCOVER, giving modules time to load.
const DISCOVER_DELAY_TICKS: u32 = 60;

pub struct ModuleDiscoveryImpl {
    emitter: EventEmitter,
    register_receptor: EventReceptor,
    ticks: Cell<u32>,
    fired: Cell<bool>,
}

impl GuestModuleDiscovery for ModuleDiscoveryImpl {
    fn new() -> Self {
        let emitter = register_emitter(None, f32::MAX, &[]);
        let register_receptor = register_receptor(&[CH_REGISTER.to_string()], None, f32::MAX, &[]);
        Self {
            emitter,
            register_receptor,
            ticks: Cell::new(0),
            fired: Cell::new(false),
        }
    }

    fn poll(&self) -> Vec<RegisteredModule> {
        if !self.fired.get() {
            let t = self.ticks.get() + 1;
            self.ticks.set(t);
            if t >= DISCOVER_DELAY_TICKS {
                self.emitter.emit(CH_DISCOVER, &[]);
                self.fired.set(true);
            }
        }

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
