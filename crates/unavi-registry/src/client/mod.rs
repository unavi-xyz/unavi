//! Following registries: one [`RegistryClient`] per registry, and the
//! [`Followed`] set the rest of the app reads them from.

use std::sync::Arc;

use anyhow::Context;
use iroh::{
    Endpoint,
    EndpointAddr,
};
use iroh_docs::NamespaceId;
use irpc::Client;
use n0_future::{
    BufferedStreamExt,
    StreamExt,
};
use time::OffsetDateTime;
use unavi_identity::{
    authorship::Authorship,
    identity::Identity,
    resolver::Resolver,
    signed::{
        Signable,
        Signed,
    },
};
use unavi_store::Store;

pub use crate::client::follow::{
    Followed,
    Target,
    resolve_batch,
};
use crate::{
    claim::{
        Presence,
        Submission,
    },
    rpc::{
        ALPN,
        Announce,
        Occupants,
        RegistryService,
        Retract,
        Submit,
        Views,
    },
    views::ViewIds,
};

mod follow;

/// Most occupant signatures verified at once.
const VERIFY_CONCURRENCY: usize = 8;

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

    fn sign<T: Signable>(&self, payload: &T) -> anyhow::Result<Signed<T>> {
        payload
            .sign(self.identity.signing_key())
            .context("sign registry payload")
    }

    /// Lists `submission.ns`, which `authorship` must prove this identity
    /// authors (see [`unavi_identity::authorship::claim`]).
    pub async fn submit(
        &self,
        submission: &Submission,
        authorship: Authorship,
    ) -> anyhow::Result<()> {
        let submission = self.sign(submission)?;

        self.client
            .rpc(Submit {
                submission,
                authorship,
            })
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

    /// Occupants of `ns`, each unexpired and verified against its announcer's
    /// DID. A registry is not trusted to have filtered them.
    pub async fn occupants(&self, ns: NamespaceId) -> anyhow::Result<Vec<Presence>> {
        let signed = self
            .client
            .rpc(Occupants { ns })
            .await?
            .map_err(|e| anyhow::anyhow!("occupants failed: {e}"))?;

        let now = OffsetDateTime::now_utc().unix_timestamp();
        let claimed = signed.into_iter().filter_map(|entry| {
            let presence = entry.payload().ok()?;
            (presence.ns == ns && presence.expires > now).then_some((entry, presence))
        });

        let resolver = &self.resolver;
        let verified = n0_future::stream::iter(claimed)
            .map(|(entry, presence)| async move {
                entry
                    .verify_did(&presence.did, resolver)
                    .await
                    .is_ok()
                    .then_some(presence)
            })
            .buffered_unordered(VERIFY_CONCURRENCY)
            .filter_map(|presence| presence)
            .collect::<Vec<_>>()
            .await;

        Ok(verified)
    }

    pub async fn views(&self) -> anyhow::Result<ViewIds> {
        let ids = self
            .client
            .rpc(Views)
            .await?
            .map_err(|e| anyhow::anyhow!("views failed: {e}"))?;

        Ok(ids)
    }

    /// Joins this registry's view docs read-only, returning their namespaces.
    /// Views are the only thing a client syncs.
    ///
    /// A view is only a cached copy, so a followed registry is joined on every
    /// sync, which keeps the retention sweep from taking it.
    pub async fn sync_views(&self, store: &Store) -> anyhow::Result<Vec<NamespaceId>> {
        let ids = self.views().await?;
        let mut synced = Vec::new();

        for ns in ids.all() {
            // Syncing and fetching outlive the handles join returns.
            let _joined = store.join(ns, vec![self.host.clone()]).await?;
            synced.push(ns);
        }

        Ok(synced)
    }

    #[must_use]
    pub const fn host(&self) -> &EndpointAddr {
        &self.host
    }
}
