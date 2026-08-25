use std::{
    str::FromStr,
    time::Duration,
};

use bevy::prelude::*;
use bevy_wds::{
    SyncTargets,
    set_registries,
    set_registry_clients,
};
use iroh::{
    Endpoint,
    EndpointAddr,
    EndpointId,
};
use unavi_registry::client::RegistryClient;
use unavi_util::async_commands::AsyncCommands;
use wds::{
    DataStore,
    WDS_SERVICE_TYPE,
    actor::Actor,
    resolve::{
        resolve,
        resolve_allowing_loopback,
    },
};
use xdid::core::did::Did;

const RETRY_DELAY: Duration = Duration::from_secs(4);
const MAX_RETRY_DELAY: Duration = Duration::from_mins(1);

/// Builds a client per followed registry, syncs each one's view docs locally,
/// and publishes both for off-world access.
///
/// A registry is a service the client consults, never one it runs; with no
/// sync targets configured there is nothing to follow.
pub async fn sync(store: &DataStore, endpoint: &Endpoint, targets: &[Actor]) {
    let mut clients = Vec::new();
    let mut namespaces = Vec::new();

    for actor in targets {
        let client = RegistryClient::new(endpoint, actor.host().clone(), actor.clone());

        match client.sync_views(store.docs()).await {
            Ok(mut views) => namespaces.append(&mut views),
            Err(err) => {
                warn!(?err, "failed syncing registry views");
                continue;
            }
        }

        clients.push(client);
    }

    set_registries(namespaces);
    set_registry_clients(clients);
}

/// Resolves every target, returning the ones that answered alongside the DIDs
/// that did not.
///
/// An unresolved target is handed to [`retry`] rather than failing the load: a
/// client with no reachable server still runs peer to peer.
pub async fn resolve_batch(
    store: &DataStore,
    dids: Vec<String>,
    allow_loopback: bool,
) -> (Vec<Actor>, Vec<String>) {
    let mut actors = Vec::new();
    let mut unresolved = Vec::new();

    for did_str in dids {
        match resolve_target(&did_str, allow_loopback).await {
            Ok(addr) => {
                info!(target = did_str, "registering WDS sync target");
                actors.push(store.remote_actor(addr));
            }
            Err(err) => {
                warn!(target = did_str, ?err, "failed to resolve WDS sync target");
                unresolved.push(did_str);
            }
        }
    }

    (actors, unresolved)
}

/// Keeps resolving the targets that were unreachable at startup, so a server
/// brought up after the client is still followed without a restart.
pub async fn retry(
    store: std::sync::Arc<DataStore>,
    endpoint: Endpoint,
    mut targets: Vec<Actor>,
    mut unresolved: Vec<String>,
    store_entity: Entity,
    allow_loopback: bool,
) {
    let mut delay = RETRY_DELAY;

    while !unresolved.is_empty() {
        n0_future::time::sleep(delay).await;
        delay = (delay * 2).min(MAX_RETRY_DELAY);

        let (actors, pending) = resolve_batch(&store, unresolved, allow_loopback).await;
        unresolved = pending;

        if actors.is_empty() {
            continue;
        }
        targets.extend(actors);

        sync(&store, &endpoint, &targets).await;

        let published = targets.clone();
        let sent = AsyncCommands::default()
            .push(move |world: &mut World| {
                if let Some(mut existing) = world.get_mut::<SyncTargets>(store_entity) {
                    existing.0 = published;
                }
            })
            .send()
            .await;

        if let Err(err) = sent {
            error!(?err, "failed to publish resolved sync targets");
            return;
        }
    }
}

async fn resolve_target(did_str: &str, allow_loopback: bool) -> anyhow::Result<EndpointAddr> {
    let did = Did::from_str(did_str)?;

    let doc = if allow_loopback {
        resolve_allowing_loopback(&did).await
    } else {
        resolve(&did).await
    }
    .ok_or_else(|| anyhow::anyhow!("could not resolve {did}"))?;

    let services = doc.service.unwrap_or_default();
    let wds = services
        .iter()
        .find(|s| s.id == "wds" && s.typ.iter().any(|t| t == WDS_SERVICE_TYPE))
        .ok_or_else(|| anyhow::anyhow!("no `wds` service in DID document"))?;

    let endpoint_str = wds
        .service_endpoint
        .first()
        .ok_or_else(|| anyhow::anyhow!("`wds` service has no serviceEndpoint"))?;

    let endpoint_id = EndpointId::from_str(endpoint_str)?;
    Ok(EndpointAddr::from(endpoint_id))
}
