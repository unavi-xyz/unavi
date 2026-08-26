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
use unavi_identity::{
    identity::Identity,
    resolve::Resolver,
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
/// The registry reads the caller's DID off the connection, proven once by
/// `wired/auth`, so nothing here carries a credential.
#[derive(Clone)]
pub struct RegistryClient {
    client:   Client<RegistryService>,
    host:     EndpointAddr,
    identity: Arc<Identity>,
    resolver: Arc<Resolver>,
}

impl RegistryClient {
    #[must_use]
    pub fn new(
        endpoint: &Endpoint,
        host: EndpointAddr,
        identity: Arc<Identity>,
        resolver: Arc<Resolver>,
    ) -> Self {
        let client = irpc_iroh::client(endpoint.clone(), host.clone(), ALPN);
        Self {
            client,
            host,
            identity,
            resolver,
        }
    }

    fn sign<T: Signable>(&self, payload: &T) -> anyhow::Result<SignedBytes<T>> {
        payload
            .sign(self.identity.signing_key())
            .context("sign registry payload")
    }

    pub async fn submit(&self, submission: &Submission) -> anyhow::Result<()> {
        let submission = self.sign(submission)?;

        self.client
            .rpc(Submit { submission })
            .await?
            .map_err(|e| anyhow::anyhow!("submit failed: {e}"))?;

        Ok(())
    }

    pub async fn retract(&self, ns: NamespaceId) -> anyhow::Result<()> {
        self.client
            .rpc(Retract { ns })
            .await?
            .map_err(|e| anyhow::anyhow!("retract failed: {e}"))?;

        Ok(())
    }

    pub async fn announce(&self, presence: &Presence) -> anyhow::Result<()> {
        let presence = self.sign(presence)?;

        self.client
            .rpc(Announce { presence })
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
            if entry.verify(&presence.did, &self.resolver).await.is_ok() {
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
        let mut synced = Vec::new();

        for ns in [ids.recent, ids.featured, ids.categories, ids.active] {
            let doc = unavi_store::namespace::ensure_open(docs, ns).await?;
            doc.start_sync(vec![self.host.clone()]).await?;
            synced.push(ns);
        }

        Ok(synced)
    }
}
