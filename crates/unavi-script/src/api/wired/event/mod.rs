use wasmtime::component::{Resource, ResourceTable};

pub mod bridge;
pub mod firewall;
pub mod registry;

pub use firewall::DocumentFirewall;
pub use registry::EventRegistry;

pub mod bindings {
    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-event",
        with: {
            "wired:scene/types.node":
                crate::api::wired::scene::node::HostNode,
            "wired:event/types.event-emitter": super::HostEventEmitter,
            "wired:event/types.event-receptor": super::HostEventReceptor,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use bindings::wired::event::types::Event as WitEvent;

#[derive(Default)]
pub struct WiredEventRt {
    pub registry: EventRegistry,
    pub table: ResourceTable,
}

pub struct HostEventEmitter {
    entity: Option<bevy::prelude::Entity>,
    radius: f32,
    sender_doc_id: Vec<u8>,
    target_documents: Vec<Vec<u8>>,
}

pub struct HostEventReceptor {
    queue: registry::ReceptorQueue,
}

use crate::load::state::RuntimeData;

impl bindings::wired::event::api::Host for RuntimeData {
    async fn register_emitter(
        &mut self,
        node: Option<Resource<crate::api::wired::scene::node::HostNode>>,
        radius: f32,
        target_documents: Vec<Vec<u8>>,
    ) -> wasmtime::Result<Resource<HostEventEmitter>> {
        let entity = if let Some(n) = node {
            let inner = std::sync::Arc::clone(&self.wired_scene.table.get(&n)?.inner);
            let e = inner
                .entity
                .lock()
                .expect("entity lock")
                .unwrap_or(bevy::prelude::Entity::PLACEHOLDER);
            Some(e)
        } else {
            None
        };
        let sender_doc_id = self.wired_scene.doc_id.as_bytes().to_vec();
        Ok(self.wired_event.table.push(HostEventEmitter {
            entity,
            radius,
            sender_doc_id,
            target_documents,
        })?)
    }

    async fn register_receptor(
        &mut self,
        channels: Vec<String>,
        node: Option<Resource<crate::api::wired::scene::node::HostNode>>,
        radius: f32,
        source_documents: Vec<Vec<u8>>,
    ) -> wasmtime::Result<Resource<HostEventReceptor>> {
        let doc_id = self.wired_scene.doc_id.as_bytes().to_vec();
        let queue = if let Some(n) = node {
            let inner = std::sync::Arc::clone(&self.wired_scene.table.get(&n)?.inner);
            let entity = inner
                .entity
                .lock()
                .expect("entity lock")
                .unwrap_or(bevy::prelude::Entity::PLACEHOLDER);
            self.wired_event
                .registry
                .0
                .lock()
                .expect("registry lock")
                .register_node(entity, channels, radius, source_documents, doc_id)
        } else {
            self.wired_event
                .registry
                .0
                .lock()
                .expect("registry lock")
                .register_global(channels, source_documents, doc_id)
        };
        Ok(self.wired_event.table.push(HostEventReceptor { queue })?)
    }
}

impl bindings::wired::event::types::Host for RuntimeData {}

impl bindings::wired::event::types::HostEventEmitter for RuntimeData {
    async fn emit(
        &mut self,
        self_: Resource<HostEventEmitter>,
        channel: String,
        payload: Vec<u8>,
    ) -> wasmtime::Result<()> {
        let emitter = self.wired_event.table.get(&self_)?;
        let emission = registry::PendingEmission {
            node: emitter.entity,
            channel,
            payload,
            radius: emitter.radius,
            sender_doc_id: emitter.sender_doc_id.clone(),
            target_documents: emitter.target_documents.clone(),
        };
        self.wired_event
            .registry
            .0
            .lock()
            .expect("registry lock")
            .push_emission(emission);
        Ok(())
    }

    async fn drop(&mut self, rep: Resource<HostEventEmitter>) -> wasmtime::Result<()> {
        self.wired_event.table.delete(rep)?;
        Ok(())
    }
}

impl bindings::wired::event::types::HostEventReceptor for RuntimeData {
    async fn poll(
        &mut self,
        self_: Resource<HostEventReceptor>,
    ) -> wasmtime::Result<Option<WitEvent>> {
        let receptor = self.wired_event.table.get(&self_)?;
        let event = receptor.queue.lock().expect("queue lock").pop_front();
        Ok(event.map(|e| WitEvent {
            channel: e.channel,
            payload: e.payload,
            sender_node: None,
            sender_document: e.sender_document,
        }))
    }

    async fn drop(&mut self, rep: Resource<HostEventReceptor>) -> wasmtime::Result<()> {
        let receptor = self.wired_event.table.delete(rep)?;
        self.wired_event
            .registry
            .0
            .lock()
            .expect("registry lock")
            .remove_receptor(&receptor.queue);
        Ok(())
    }
}
