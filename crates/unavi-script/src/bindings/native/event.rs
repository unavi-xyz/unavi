//! `wired:event`.

use wasmtime::component::Resource;

use super::{
    HostCtx,
    convert,
    generated::wired::{
        core::ids::DocumentId,
        event::messaging::{
            Host,
            HostMessageSubscription,
            Message,
            Scope,
        },
    },
};
use crate::{
    error::ScriptError,
    host::event::{
        self,
        MessageSubscription,
    },
};

fn scope(scope: Scope) -> event::Scope {
    match scope {
        Scope::Global => event::Scope::Global,
        Scope::Spatial(spatial) => {
            let (doc, prim) = convert::prim_ref(spatial.origin);
            event::Scope::Spatial {
                doc,
                prim,
                radius: spatial.radius,
            }
        }
    }
}

fn docs(docs: Vec<DocumentId>) -> Vec<hsd::id::DocId> {
    docs.into_iter().map(convert::doc_id).collect()
}

impl Host for HostCtx {
    fn emit(
        &mut self,
        channel: String,
        payload: Vec<u8>,
        audience: Option<Vec<DocumentId>>,
        scope: Scope,
    ) -> Result<(), ScriptError> {
        if audience
            .as_ref()
            .is_some_and(|docs| docs.len() > event::MAX_FILTER_DOCUMENTS)
        {
            return Err(ScriptError::invalid("a filter names at most 64 documents"));
        }
        event::emit(
            &self.host,
            &channel,
            payload,
            audience.map(docs),
            self::scope(scope),
        )
    }

    fn listen(
        &mut self,
        channels: Vec<String>,
        senders: Option<Vec<DocumentId>>,
        scope: Scope,
    ) -> Result<Resource<MessageSubscription>, ScriptError> {
        event::listen(
            &mut self.host,
            channels,
            senders.map(docs),
            self::scope(scope),
        )
        .map(Resource::new_own)
    }
}

impl HostMessageSubscription for HostCtx {
    fn drain(
        &mut self,
        subscription: Resource<MessageSubscription>,
        max: u32,
    ) -> wasmtime::Result<Vec<Message>> {
        Ok(event::drain(&mut self.host, subscription.rep(), max)?
            .into_iter()
            .map(|message| Message {
                channel:     message.channel,
                payload:     message.payload,
                sender:      Some(convert::wit_doc_id(message.sender)),
                distance:    message.distance,
                sent_at:     message.sent_at,
                claim_token: message.claim_token,
            })
            .collect())
    }

    fn dropped(&mut self, subscription: Resource<MessageSubscription>) -> wasmtime::Result<u64> {
        Ok(event::dropped(&self.host, subscription.rep())?)
    }

    fn claim(
        &mut self,
        subscription: Resource<MessageSubscription>,
        token: u64,
    ) -> wasmtime::Result<bool> {
        Ok(event::claim(&self.host, subscription.rep(), token)?)
    }

    fn drop(&mut self, subscription: Resource<MessageSubscription>) -> wasmtime::Result<()> {
        Ok(event::close(&mut self.host, subscription.rep())?)
    }
}
