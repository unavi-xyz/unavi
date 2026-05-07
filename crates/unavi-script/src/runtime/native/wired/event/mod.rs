use wasmtime::component::Resource;

use crate::runtime::{
    Runtime,
    shared::{
        self,
        wired::event::{EventFilter, EventReceptorRes, EventScope},
    },
};

pub mod bindings {
    pub use crate::runtime::shared::wired::{event::EventReceptorRes, scene::node::NodeRes};

    wasmtime::component::bindgen!({
        path: "../../protocol/wit/wired-event",
        with: {
            "wired:event/types.event-receptor": EventReceptorRes,
            "wired:scene/types.node": NodeRes,
        },
        imports: { default: async | trappable },
        exports: { default: async | trappable },
    });
}

use bindings::wired::event::{
    api::EventReceptor,
    types::{
        Event, EventFilter as WitFilter, EventScope as WitScope, EventSender, HostEventReceptor,
    },
};

impl From<WitScope> for EventScope {
    fn from(s: WitScope) -> Self {
        match s {
            WitScope::Global => Self::Global,
            WitScope::Spatial(r) => Self::Spatial(r),
        }
    }
}

fn wit_filter_to_shared(f: WitFilter, node_rep: Option<u32>) -> EventFilter {
    EventFilter {
        node: node_rep,
        scope: f.scope.into(),
        documents: f.documents,
    }
}

impl From<shared::wired::event::EventSender> for EventSender {
    fn from(s: shared::wired::event::EventSender) -> Self {
        match s {
            shared::wired::event::EventSender::Global => Self::Global,
            shared::wired::event::EventSender::Spatial => Self::Spatial,
        }
    }
}

impl bindings::wired::event::types::Host for Runtime {}

impl HostEventReceptor for Runtime {
    async fn poll(&mut self, self_: Resource<EventReceptorRes>) -> wasmtime::Result<Option<Event>> {
        shared::wired::event::receptor_poll(&self.api, self_.rep())
            .map(|opt| {
                opt.map(|e| Event {
                    channel: e.channel,
                    payload: e.payload,
                    sender: e.sender.into(),
                    sender_document: e.sender_document,
                    time: e.time,
                })
            })
            .map_err(wasmtime::Error::from_anyhow)
    }

    async fn drop(&mut self, rep: Resource<EventReceptorRes>) -> wasmtime::Result<()> {
        shared::wired::event::receptor_drop(&self.api, rep.rep())
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
        let node_rep = filter.node.as_ref().map(Resource::rep);
        shared::wired::event::emit(
            &self.api,
            channel,
            payload,
            wit_filter_to_shared(filter, node_rep),
        )
        .map_err(wasmtime::Error::from_anyhow)
    }

    async fn listen(
        &mut self,
        channels: Vec<String>,
        filter: WitFilter,
    ) -> wasmtime::Result<Resource<EventReceptor>> {
        let node_rep = filter.node.as_ref().map(Resource::rep);
        shared::wired::event::listen(&self.api, channels, wit_filter_to_shared(filter, node_rep))
            .map(Resource::new_own)
            .map_err(wasmtime::Error::from_anyhow)
    }
}
