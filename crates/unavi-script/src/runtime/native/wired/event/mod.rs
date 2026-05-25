use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    shared::{
        self,
        registry::{
            event::SenderScope,
            transform::AbsoluteNodeId,
        },
        wired::{
            event::{
                EventFilter,
                EventReceptorRes,
                EventScope,
            },
            scene::prim::PrimRes,
        },
    },
};

pub mod bindings {
    pub use crate::runtime::shared::wired::{
        event::EventReceptorRes,
        scene::prim::PrimRes,
    };

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-event",
        with: {
            "wired:event/types.event-receptor": EventReceptorRes,
            "wired:scene/types.prim": PrimRes,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use bindings::wired::event::{
    api::EventReceptor,
    types::{
        Event,
        EventFilter as WitFilter,
        EventScope as WitScope,
        EventSender,
        HostEventReceptor,
        SenderScope as WitSenderScope,
        SpatialSender,
    },
};

fn wit_filter_to_shared(f: WitFilter) -> EventFilter {
    EventFilter {
        documents: f.documents,
        scope:     match f.scope {
            WitScope::Global => EventScope::Global,
            WitScope::Spatial(v) => EventScope::Spatial {
                node:   v.prim.rep(),
                radius: v.radius,
            },
        },
    }
}

impl bindings::wired::event::types::Host for Runtime {}

impl HostEventReceptor for Runtime {
    async fn poll(&mut self, self_: Resource<EventReceptorRes>) -> wasmtime::Result<Option<Event>> {
        let Some(event) = shared::wired::event::receptor_poll(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?
        else {
            return Ok(None);
        };

        let scope = match event.sender_scope {
            SenderScope::Global => WitSenderScope::Global,
            SenderScope::Spatial {
                distance,
                node: AbsoluteNodeId { doc: doc_id, node },
            } => {
                let prim_rep = self.api.wired_scene.lock().await.prims.insert(PrimRes {
                    doc: std::sync::Arc::clone(&self.api.doc),
                    doc_id,
                    id: node,
                    is_proxy: true,
                });
                WitSenderScope::Spatial(SpatialSender {
                    distance,
                    prim: Resource::new_own(prim_rep),
                })
            }
        };

        Ok(Some(Event {
            channel: event.channel,
            payload: event.payload.as_ref().clone(),
            sender:  EventSender {
                document: event.sender_document,
                scope,
            },
            time:    event.time,
        }))
    }

    async fn drop(&mut self, rep: Resource<EventReceptorRes>) -> wasmtime::Result<()> {
        shared::wired::event::receptor_drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}

impl bindings::wired::event::api::Host for Runtime {
    async fn emit(
        &mut self,
        channel: String,
        payload: Vec<u8>,
        filter: WitFilter,
    ) -> wasmtime::Result<()> {
        shared::wired::event::emit(&self.api, channel, payload, wit_filter_to_shared(filter))
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn listen(
        &mut self,
        channels: Vec<String>,
        filter: WitFilter,
    ) -> wasmtime::Result<Resource<EventReceptor>> {
        shared::wired::event::listen(&self.api, channels, wit_filter_to_shared(filter))
            .await
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }
}
