//! `wired:portal`.

use wasmtime::component::Resource;

use super::{
    HostCtx,
    convert,
    generated::wired::{
        core::ids::{
            DocumentId,
            PrimId,
        },
        portal::portals::{
            Host,
            HostIntentSubscription,
            LinkIntent,
        },
    },
};
use crate::{
    error::ScriptError,
    host::{
        portal::{
            self,
            IntentSubscription,
        },
        scene::DocumentRes,
        shared_state::link_intents,
    },
};

fn intent(intent: LinkIntent) -> link_intents::LinkIntent {
    link_intents::LinkIntent {
        source_space: convert::doc_id(intent.source_space),
        link:         convert::link_id(intent.link),
    }
}

impl Host for HostCtx {
    async fn open(
        &mut self,
        doc: Resource<DocumentRes>,
        prim: PrimId,
        target_space: DocumentId,
    ) -> Result<(), ScriptError> {
        portal::open(
            &self.host,
            doc.rep(),
            convert::prim_id(prim),
            convert::doc_id(target_space),
        )
        .await
    }

    async fn pair(
        &mut self,
        doc: Resource<DocumentRes>,
        prim: PrimId,
        link: LinkIntent,
    ) -> Result<(), ScriptError> {
        portal::pair(&self.host, doc.rep(), convert::prim_id(prim), intent(link)).await
    }

    async fn travel(&mut self, target_space: DocumentId) -> Result<(), ScriptError> {
        portal::travel(&self.host, convert::doc_id(target_space)).await
    }

    fn intents(&mut self) -> Result<Resource<IntentSubscription>, ScriptError> {
        portal::intents(&mut self.host).map(Resource::new_own)
    }
}

impl HostIntentSubscription for HostCtx {
    fn drain(
        &mut self,
        subscription: Resource<IntentSubscription>,
        max: u32,
    ) -> wasmtime::Result<Vec<LinkIntent>> {
        Ok(portal::drain(&mut self.host, subscription.rep(), max)?
            .into_iter()
            .map(|intent| LinkIntent {
                source_space: convert::wit_doc_id(intent.source_space),
                link:         convert::wit_link_id(intent.link),
            })
            .collect())
    }

    fn dropped(&mut self, subscription: Resource<IntentSubscription>) -> wasmtime::Result<u64> {
        Ok(portal::dropped(&self.host, subscription.rep())?)
    }

    fn claim(
        &mut self,
        subscription: Resource<IntentSubscription>,
        link: LinkIntent,
    ) -> wasmtime::Result<bool> {
        Ok(portal::claim(&self.host, subscription.rep(), intent(link))?)
    }

    fn drop(&mut self, subscription: Resource<IntentSubscription>) -> wasmtime::Result<()> {
        Ok(portal::close(&mut self.host, subscription.rep())?)
    }
}
