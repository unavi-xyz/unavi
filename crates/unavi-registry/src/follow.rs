//! Following a registry: turning configured DIDs into clients, and publishing
//! both the clients and the view docs they sync for off-world access.

use std::{
    str::FromStr,
    sync::Arc,
};

use iroh::{
    Endpoint,
    EndpointAddr,
    EndpointId,
};
use iroh_docs::NamespaceId;
use parking_lot::RwLock;
use tracing::{
    info,
    warn,
};
use unavi_identity::{
    ENDPOINT_SERVICE_ID,
    ENDPOINT_SERVICE_TYPE,
    identity::Identity,
    resolve::Resolver,
};
use unavi_store::store::Store;
use xdid::core::did::Did;

use crate::client::RegistryClient;

/// View docs of the registries this node follows. Views are the curated docs a
/// registry publishes, and the only thing a client syncs from one.
static REGISTRIES: RwLock<Vec<NamespaceId>> = RwLock::new(Vec::new());

static REGISTRY_CLIENTS: RwLock<Vec<RegistryClient>> = RwLock::new(Vec::new());

#[must_use]
pub fn registries() -> Vec<NamespaceId> {
    REGISTRIES.read().clone()
}

#[must_use]
pub fn registry_clients() -> Vec<RegistryClient> {
    REGISTRY_CLIENTS.read().clone()
}

/// Builds a client per followed registry, syncs each one's view docs locally,
/// and publishes both.
///
/// A registry is a service the client consults, never one it runs; with no
/// targets configured there is nothing to follow.
pub async fn sync(
    store: &Store,
    endpoint: &Endpoint,
    targets: &[EndpointAddr],
    identity: Arc<Identity>,
    resolver: Arc<Resolver>,
) {
    let mut clients = Vec::new();
    let mut namespaces = Vec::new();

    for host in targets {
        let client = RegistryClient::new(
            endpoint,
            host.clone(),
            Arc::clone(&identity),
            Arc::clone(&resolver),
        );

        match client.sync_views(store).await {
            Ok(mut views) => namespaces.append(&mut views),
            Err(err) => {
                warn!(?err, "failed syncing registry views");
                continue;
            }
        }

        clients.push(client);
    }

    *REGISTRIES.write() = namespaces;
    *REGISTRY_CLIENTS.write() = clients;
}

/// Resolves every target, returning the ones that answered alongside the DIDs
/// that did not.
///
/// An unresolved target is worth retrying rather than failing the load over. A
/// client with no reachable server still runs peer to peer.
pub async fn resolve_batch(
    dids: Vec<String>,
    resolver: &Resolver,
) -> (Vec<EndpointAddr>, Vec<String>) {
    let mut addrs = Vec::new();
    let mut unresolved = Vec::new();

    for did_str in dids {
        match resolve_target(&did_str, resolver).await {
            Ok(addr) => {
                info!(target = did_str, "following registry");
                addrs.push(addr);
            }
            Err(err) => {
                warn!(target = did_str, ?err, "failed to resolve registry");
                unresolved.push(did_str);
            }
        }
    }

    (addrs, unresolved)
}

async fn resolve_target(did_str: &str, resolver: &Resolver) -> anyhow::Result<EndpointAddr> {
    let did = Did::from_str(did_str)?;
    let doc = resolver.resolve(&did).await?;

    let services = doc.service.unwrap_or_default();
    let service = services
        .iter()
        .find(|s| s.id == ENDPOINT_SERVICE_ID && s.typ.iter().any(|t| t == ENDPOINT_SERVICE_TYPE))
        .ok_or_else(|| anyhow::anyhow!("no `{ENDPOINT_SERVICE_ID}` service in DID document"))?;

    let endpoint_str = service
        .service_endpoint
        .first()
        .ok_or_else(|| anyhow::anyhow!("`{ENDPOINT_SERVICE_ID}` service has no serviceEndpoint"))?;

    let endpoint_id = EndpointId::from_str(endpoint_str)?;
    Ok(EndpointAddr::from(endpoint_id))
}
