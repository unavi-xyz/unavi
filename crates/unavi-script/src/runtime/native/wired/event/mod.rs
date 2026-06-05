use std::sync::Arc;

use wasmtime::component::Resource;

use crate::{
    error::ScriptError,
    permissions::ApiName,
    runtime::{
        Runtime,
        native::wired::error::Error,
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
                    EventRes,
                    EventScope,
                },
                scene::prim::PrimRes,
            },
        },
    },
};

pub mod bindings {
    pub use crate::runtime::shared::wired::{
        event::{
            EventReceptorRes,
            EventRes,
        },
        scene::prim::PrimRes,
    };

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-event",
        with: {
            "wired:event/types.event": EventRes,
            "wired:event/types.event-receptor": EventReceptorRes,
            "wired:scene/types.prim": PrimRes,
            "wired:error/types": crate::runtime::native::wired::error::types,
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
        HostEvent,
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

impl HostEvent for Runtime {
    async fn channel(&mut self, self_: Resource<EventRes>) -> wasmtime::Result<String> {
        Ok(
            shared::wired::event::event_clone_inner(&self.api, self_.rep())
                .await
                .map_err(wasmtime::Error::from_anyhow)?
                .channel,
        )
    }

    async fn payload(&mut self, self_: Resource<EventRes>) -> wasmtime::Result<Vec<u8>> {
        let inner = shared::wired::event::event_clone_inner(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?;
        Ok(inner.payload.as_ref().clone())
    }

    async fn sender(&mut self, self_: Resource<EventRes>) -> wasmtime::Result<EventSender> {
        let inner = shared::wired::event::event_clone_inner(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?;
        let scope = match inner.sender_scope {
            SenderScope::Global => WitSenderScope::Global,
            SenderScope::Spatial {
                distance,
                node: AbsoluteNodeId { doc: doc_id, node },
            } => {
                let prim_rep = self
                    .api
                    .wired_scene
                    .lock()
                    .await
                    .prims
                    .insert(
                        PrimRes {
                            doc: Arc::clone(&self.api.doc),
                            doc_id,
                            id: node,
                            is_proxy: true,
                        },
                        &self.api.quota,
                    )
                    .map_err(wasmtime::Error::from)?;
                WitSenderScope::Spatial(SpatialSender {
                    distance,
                    prim: Resource::new_own(prim_rep),
                })
            }
        };
        Ok(EventSender {
            document: inner.sender_document,
            scope,
        })
    }

    async fn time(&mut self, self_: Resource<EventRes>) -> wasmtime::Result<u64> {
        Ok(
            shared::wired::event::event_clone_inner(&self.api, self_.rep())
                .await
                .map_err(wasmtime::Error::from_anyhow)?
                .time,
        )
    }

    async fn consume(&mut self, self_: Resource<EventRes>) -> wasmtime::Result<bool> {
        shared::wired::event::event_consume(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<EventRes>) -> wasmtime::Result<()> {
        shared::wired::event::event_drop(&self.api, rep.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)
    }
}

impl HostEventReceptor for Runtime {
    async fn poll(
        &mut self,
        self_: Resource<EventReceptorRes>,
    ) -> wasmtime::Result<Option<Resource<Event>>> {
        let Some(event) = shared::wired::event::receptor_poll(&self.api, self_.rep())
            .await
            .map_err(wasmtime::Error::from_anyhow)?
        else {
            return Ok(None);
        };
        let rep = shared::wired::event::insert_event(&self.api, event)
            .await
            .map_err(wasmtime::Error::from)?;
        Ok(Some(Resource::new_own(rep)))
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
    ) -> wasmtime::Result<Result<(), Error>> {
        if let Err(err) = self.api.require(ApiName::Event) {
            return Ok(Err(err.into()));
        }
        Ok(
            shared::wired::event::emit(&self.api, channel, payload, wit_filter_to_shared(filter))
                .await
                .map_err(|err| ScriptError::from(err).into()),
        )
    }

    async fn listen(
        &mut self,
        channels: Vec<String>,
        filter: WitFilter,
    ) -> wasmtime::Result<Result<Resource<EventReceptor>, Error>> {
        if let Err(err) = self.api.require(ApiName::Event) {
            return Ok(Err(err.into()));
        }
        Ok(
            shared::wired::event::listen(&self.api, channels, wit_filter_to_shared(filter))
                .await
                .map(Resource::new_own)
                .map_err(|err| ScriptError::from(err).into()),
        )
    }
}
