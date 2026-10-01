//! Which registries this node follows, resolved from their DIDs.

use std::{
    str::FromStr,
    sync::Arc,
};

use iroh::{
    Endpoint,
    EndpointAddr,
};
use iroh_docs::NamespaceId;
use parking_lot::RwLock;
use tracing::{
    info,
    warn,
};
use unavi_identity::{
    auth::Bindings,
    did_document,
    identity::Identity,
    resolver::Resolver,
};
use unavi_store::Store;
use xdid::core::did::Did;

use crate::client::RegistryClient;

/// A registry to follow: the DID it is known by and the endpoint that DID
/// names.
#[derive(Debug, Clone)]
pub struct Target {
    pub did:  Did,
    pub addr: EndpointAddr,
}

/// The registries this node follows, and the view docs each publishes. Clones
/// share one set.
///
/// Empty until [`Self::sync`] first completes.
#[derive(Clone, Default)]
pub struct Followed(Arc<RwLock<Inner>>);

#[derive(Default)]
struct Inner {
    views:   Vec<NamespaceId>,
    clients: Vec<RegistryClient>,
}

impl Followed {
    /// The view docs of every followed registry.
    #[must_use]
    pub fn views(&self) -> Vec<NamespaceId> {
        self.0.read().views.clone()
    }

    #[must_use]
    pub fn is_view(&self, ns: NamespaceId) -> bool {
        self.0.read().views.contains(&ns)
    }

    #[must_use]
    pub fn clients(&self) -> Vec<RegistryClient> {
        self.0.read().clients.clone()
    }

    /// Builds a client per target, syncs each one's views, and replaces the
    /// followed set with those that answered.
    ///
    /// A target is followed only if its endpoint proved, over `wired/auth`, the
    /// DID it was resolved from, so a DID document cannot point this node at
    /// someone else's registry.
    pub async fn sync(
        &self,
        store: &Store,
        endpoint: &Endpoint,
        bindings: &Bindings,
        targets: &[Target],
        identity: &Arc<Identity>,
        resolver: &Arc<Resolver>,
    ) {
        let mut clients = Vec::new();
        let mut views = Vec::new();

        for target in targets {
            let client = RegistryClient::new(
                endpoint,
                target.addr.clone(),
                Arc::clone(identity),
                Arc::clone(resolver),
            );

            let synced = match client.sync_views(store).await {
                Ok(synced) => synced,
                Err(err) => {
                    warn!(did = %target.did, ?err, "failed syncing registry views");
                    continue;
                }
            };

            if bindings.did_of(target.addr.id).as_ref() != Some(&target.did) {
                warn!(did = %target.did, "registry did not prove the DID it was followed by");
                continue;
            }

            views.extend(synced);
            clients.push(client);
        }

        *self.0.write() = Inner { views, clients };
    }
}

/// Resolves every target, returning the ones that answered alongside the DIDs
/// that did not.
///
/// An unresolved target is worth retrying rather than failing the load over. A
/// client with no reachable server still runs peer to peer.
pub async fn resolve_batch(dids: Vec<String>, resolver: &Resolver) -> (Vec<Target>, Vec<String>) {
    let mut targets = Vec::new();
    let mut unresolved = Vec::new();

    for did in dids {
        match resolve_target(&did, resolver).await {
            Ok(target) => {
                info!(%did, "following registry");
                targets.push(target);
            }
            Err(err) => {
                warn!(%did, ?err, "failed to resolve registry");
                unresolved.push(did);
            }
        }
    }

    (targets, unresolved)
}

async fn resolve_target(did: &str, resolver: &Resolver) -> anyhow::Result<Target> {
    let did = Did::from_str(did)?;
    let doc = resolver.resolve(&did).await?;
    let endpoint = did_document::endpoint_of(&doc)
        .ok_or_else(|| anyhow::anyhow!("DID document names no iroh endpoint"))?;

    Ok(Target {
        did,
        addr: EndpointAddr::from(endpoint),
    })
}
