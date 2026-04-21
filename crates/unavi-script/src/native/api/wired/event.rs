use std::sync::Arc;

use blake3::Hash;
use wasmtime::component::{Resource, ResourceTable};

use crate::{
    event_registry::{EventRegistry, PendingEmission, ReceptorQueue},
    load::native::state::RuntimeData,
    native::api::wired::scene::node::HostNode,
};

pub mod bindings {
    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-event",
        with: {
            "wired:scene/types.node":
                crate::native::api::wired::scene::node::HostNode,
            "wired:event/types.event-receptor": super::HostEventReceptor,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use bindings::wired::event::types::{Event as WitEvent, EventScope as WitEventScope, EventSender};

#[derive(Default)]
pub struct WiredEventRt {
    pub registry: EventRegistry,
    pub table: ResourceTable,
}

pub struct HostEventReceptor {
    queue: ReceptorQueue,
}

impl bindings::wired::event::api::Host for RuntimeData {
    async fn emit(
        &mut self,
        channel: String,
        payload: Vec<u8>,
        filter: bindings::wired::event::types::EventFilter,
    ) -> wasmtime::Result<()> {
        let (node, radius) = scope_to_node(&mut self.wired_scene.table, filter.node, filter.scope)?;
        let target_documents = parse_documents(filter.documents)?;
        let emission = PendingEmission {
            node,
            channel,
            payload,
            radius,
            sender_doc_id: self.wired_scene.doc_id,
            target_documents,
        };
        self.wired_event
            .registry
            .0
            .lock()
            .expect("registry lock")
            .push_emission(emission);
        Ok(())
    }

    async fn listen(
        &mut self,
        channels: Vec<String>,
        filter: bindings::wired::event::types::EventFilter,
    ) -> wasmtime::Result<Resource<HostEventReceptor>> {
        let (node, radius) = scope_to_node(&mut self.wired_scene.table, filter.node, filter.scope)?;
        let source_documents = parse_documents(filter.documents)?;
        let doc_id = self.wired_scene.doc_id;
        let queue = {
            let mut inner = self.wired_event.registry.0.lock().expect("registry lock");
            if let Some(n) = node {
                inner.register_node(n, channels, radius, source_documents, doc_id)
            } else {
                inner.register_global(channels, source_documents, doc_id)
            }
        };
        Ok(self.wired_event.table.push(HostEventReceptor { queue })?)
    }
}

impl bindings::wired::event::types::Host for RuntimeData {}

impl bindings::wired::event::types::HostEventReceptor for RuntimeData {
    async fn poll(
        &mut self,
        self_: Resource<HostEventReceptor>,
    ) -> wasmtime::Result<Option<WitEvent>> {
        let receptor = self.wired_event.table.get(&self_)?;
        let Some(event) = receptor.queue.lock().expect("queue lock").pop_front() else {
            return Ok(None);
        };
        // TODO: expose sender node handle once cross-script node sharing is supported.
        let sender = if event.sender_node.is_some() {
            EventSender::Spatial
        } else {
            EventSender::Global
        };
        Ok(Some(WitEvent {
            channel: event.channel,
            payload: event.payload,
            sender,
            sender_document: event.sender_document.as_bytes().to_vec(),
            time: event.time,
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

fn scope_to_node(
    scene_table: &mut ResourceTable,
    node: Option<Resource<HostNode>>,
    scope: WitEventScope,
) -> wasmtime::Result<(Option<Arc<bevy_hsd::cache::NodeInner>>, f32)> {
    match (node, scope) {
        (Some(n), WitEventScope::Spatial(radius)) => {
            let host_node = scene_table.delete(n)?;
            Ok((Some(Arc::clone(&host_node.inner)), radius))
        }
        _ => Ok((None, 0.0)),
    }
}

fn parse_documents(docs: Option<Vec<Vec<u8>>>) -> wasmtime::Result<Option<Vec<Hash>>> {
    match docs {
        None => Ok(None),
        Some(list) => {
            let hashes = list
                .into_iter()
                .map(|bytes| Hash::from_slice(&bytes))
                .collect::<Result<_, _>>()?;
            Ok(Some(hashes))
        }
    }
}
