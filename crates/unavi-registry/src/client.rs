use std::sync::Arc;

use anyhow::Context;
use iroh::{
    Endpoint,
    EndpointAddr,
};
use iroh_docs::{
    NamespaceId,
    protocol::Docs,
};
use irpc::Client;
use wds::{
    actor::Actor,
    identity::Identity,
    signed_bytes::{
        Signable,
        SignedBytes,
    },
};

use crate::{
    control::{
        ALPN,
        Announce,
        Occupants,
        RegistryService,
        Retract,
        Submit,
        Views,
    },
    entry::{
        Presence,
        Submission,
    },
    views::ViewIds,
};

/// Client handle for one registry.
///
/// Authentication rides on the node's existing WDS session.
#[derive(Clone)]
pub struct RegistryClient {
    actor:    Actor,
    client:   Client<RegistryService>,
    identity: Arc<Identity>,
}

impl RegistryClient {
    #[must_use]
    pub fn new(endpoint: &Endpoint, host: EndpointAddr, actor: Actor) -> Self {
        let identity = Arc::clone(actor.identity());
        let client = irpc_iroh::client(endpoint.clone(), host, ALPN);
        Self {
            actor,
            client,
            identity,
        }
    }

    /// Wraps an in-process registry, skipping the network entirely.
    #[must_use]
    pub fn local(client: Client<RegistryService>, actor: Actor) -> Self {
        let identity = Arc::clone(actor.identity());
        Self {
            actor,
            client,
            identity,
        }
    }

    fn sign<T: Signable>(&self, payload: &T) -> anyhow::Result<SignedBytes<T>> {
        payload
            .sign(self.identity.signing_key())
            .context("sign registry payload")
    }

    pub async fn submit(&self, submission: &Submission) -> anyhow::Result<()> {
        let s = self.actor.session().await.context("auth")?;
        let submission = self.sign(submission)?;

        self.client
            .rpc(Submit { s, submission })
            .await?
            .map_err(|e| anyhow::anyhow!("submit failed: {e}"))?;

        Ok(())
    }

    pub async fn retract(&self, ns: NamespaceId) -> anyhow::Result<()> {
        let s = self.actor.session().await.context("auth")?;

        self.client
            .rpc(Retract { s, ns })
            .await?
            .map_err(|e| anyhow::anyhow!("retract failed: {e}"))?;

        Ok(())
    }

    pub async fn announce(&self, presence: &Presence) -> anyhow::Result<()> {
        let s = self.actor.session().await.context("auth")?;
        let presence = self.sign(presence)?;

        self.client
            .rpc(Announce { s, presence })
            .await?
            .map_err(|e| anyhow::anyhow!("announce failed: {e}"))?;

        Ok(())
    }

    /// Occupants of a namespace, verified against each announcer's DID.
    pub async fn occupants(&self, ns: NamespaceId) -> anyhow::Result<Vec<Presence>> {
        let signed = self
            .client
            .rpc(Occupants { ns })
            .await?
            .map_err(|e| anyhow::anyhow!("occupants failed: {e}"))?;

        let mut out = Vec::new();
        for entry in signed {
            let Ok(presence) = entry.payload() else {
                continue;
            };
            if wds::signed_bytes::verify_did_signature(&entry, &presence.did).await {
                out.push(presence);
            }
        }

        Ok(out)
    }

    pub async fn views(&self) -> anyhow::Result<ViewIds> {
        let ids = self
            .client
            .rpc(Views)
            .await?
            .map_err(|e| anyhow::anyhow!("views failed: {e}"))?;

        Ok(ids)
    }

    /// Imports this registry's view docs read-only and starts syncing them,
    /// returning their namespaces. Views are the only thing a client syncs.
    pub async fn sync_views(&self, docs: &Docs) -> anyhow::Result<Vec<NamespaceId>> {
        let ids = self.views().await?;
        let host = self.actor.host().clone();
        let mut synced = Vec::new();

        for ns in [ids.recent, ids.featured, ids.categories, ids.active] {
            let doc = wds::docs::ensure_open(docs, ns).await?;
            doc.start_sync(vec![host.clone()]).await?;
            synced.push(ns);
        }

        Ok(synced)
    }
}
