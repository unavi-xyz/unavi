use std::{
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use bevy::prelude::*;
use bevy_iroh::{
    endpoint::IrohEndpoint,
    router::{
        RouterBuilderFn,
        RouterBuilderFnTarget,
    },
};
use bevy_wds::{
    LocalActor,
    LocalBlobs,
    LocalDocs,
    LocalGossip,
    SyncTargets,
    set_local_actor,
    set_registries,
    set_registry_clients,
    set_root_doc,
};
use iroh::{
    Endpoint,
    EndpointAddr,
    EndpointId,
};
use unavi_registry::client::RegistryClient;
use unavi_util::{
    async_commands::AsyncCommands,
    async_task::spawn_async_task,
};
use wds::{
    DataStore,
    WDS_SERVICE_TYPE,
    actor::Actor,
    identity::Identity,
    resolve::{
        resolve,
        resolve_allowing_loopback,
    },
};
use xdid::{
    core::did::Did,
    methods::key::keys::{
        DidKeyPair,
        PublicKey,
        p256::P256KeyPair,
    },
};

const RETRY_DELAY: Duration = Duration::from_secs(4);
const MAX_RETRY_DELAY: Duration = Duration::from_mins(1);

pub fn spawn_actors(trigger: On<Add, IrohEndpoint>, endpoints: Query<&IrohEndpoint>) {
    let entity = trigger.entity;

    let endpoint = endpoints
        .get(entity)
        .map(|e| e.0.clone())
        .expect("endpoint");

    spawn_async_task(async move {
        let mut delay_secs = 4;

        loop {
            if let Err(err) = load_store(endpoint.clone(), entity).await {
                error!(?err, "Failed to load data store");
                n0_future::time::sleep(Duration::from_secs(delay_secs)).await;
                delay_secs = delay_secs.wrapping_mul(2);
                continue;
            }
            break;
        }

        // Keep data store alive.
        // TODO merge with shared thread?
        std::future::pending::<()>().await;
    });
}

async fn load_store(endpoint: Endpoint, entity: Entity) -> anyhow::Result<()> {
    // TODO load identity from disk / browser storage
    let signing_key = P256KeyPair::generate();
    let did = signing_key.public().to_did();
    let identity = Arc::new(Identity::new(did, signing_key));

    let (store, f) = DataStore::builder(endpoint.clone())
        .gc_timer(Duration::from_mins(15))
        .build()
        .await?;
    let store = Arc::new(store);

    store.set_user_identity(Arc::clone(&identity));
    let actor = store.local_actor(Arc::clone(&identity));
    set_local_actor(actor.clone());

    if let Ok(root) = wds::kv::create(store.docs()).await {
        set_root_doc(root);
    }

    let (sync_targets, unresolved) = resolve_sync_targets(&store, &identity).await;
    sync_registries(&store, &endpoint, &sync_targets).await;

    let store_entity = AsyncCommands::default()
        .spawn((RouterBuilderFnTarget(entity), RouterBuilderFn(Some(f))))
        .send_spawn((
            LocalActor(actor),
            LocalBlobs(store.blobs().blobs().clone()),
            LocalDocs(store.docs().clone()),
            LocalGossip(store.gossip().clone()),
            SyncTargets(sync_targets.clone()),
        ))
        .await;

    if !unresolved.is_empty() {
        spawn_async_task(retry_sync_targets(
            store,
            endpoint,
            identity,
            sync_targets,
            unresolved,
            store_entity,
        ));
    }

    Ok(())
}

/// Builds a client per followed registry, syncs each one's view docs locally,
/// and publishes both for off-world access.
///
/// A registry is a service the client consults, never one it runs; with no
/// sync targets configured there is nothing to follow.
async fn sync_registries(store: &DataStore, endpoint: &Endpoint, targets: &[Actor]) {
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

/// Resolves every configured sync target, returning the ones that answered
/// alongside the DIDs that did not.
///
/// An unresolved target is handed to [`retry_sync_targets`] rather than failing
/// the load: a client with no reachable server still runs peer to peer.
async fn resolve_sync_targets(
    store: &DataStore,
    identity: &Arc<Identity>,
) -> (Vec<Actor>, Vec<String>) {
    let Ok(raw) = std::env::var("UNAVI_SYNC_TARGETS") else {
        return (Vec::new(), Vec::new());
    };

    let dids = raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();

    if dids.is_empty() {
        return (Vec::new(), Vec::new());
    }

    resolve_batch(store, identity, dids).await
}

async fn resolve_batch(
    store: &DataStore,
    identity: &Arc<Identity>,
    dids: Vec<String>,
) -> (Vec<Actor>, Vec<String>) {
    let mut actors = Vec::new();
    let mut unresolved = Vec::new();

    for did_str in dids {
        match resolve_sync_target(&did_str).await {
            Ok(addr) => {
                info!(target = did_str, "registering WDS sync target");
                actors.push(store.remote_actor(Arc::clone(identity), addr));
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
async fn retry_sync_targets(
    store: Arc<DataStore>,
    endpoint: Endpoint,
    identity: Arc<Identity>,
    mut targets: Vec<Actor>,
    mut unresolved: Vec<String>,
    store_entity: Entity,
) {
    let mut delay = RETRY_DELAY;

    while !unresolved.is_empty() {
        n0_future::time::sleep(delay).await;
        delay = (delay * 2).min(MAX_RETRY_DELAY);

        let (actors, pending) = resolve_batch(&store, &identity, unresolved).await;
        unresolved = pending;

        if actors.is_empty() {
            continue;
        }
        targets.extend(actors);

        sync_registries(&store, &endpoint, &targets).await;

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

/// Sync targets are named by the operator rather than by a peer, so a loopback
/// `did:web` is a local server they chose to run, not an SSRF probe.
fn loopback_targets_allowed() -> bool {
    std::env::var_os("UNAVI_ALLOW_LOOPBACK_SYNC_TARGETS").is_some()
}

async fn resolve_sync_target(did_str: &str) -> anyhow::Result<EndpointAddr> {
    let did = Did::from_str(did_str)?;

    let doc = if loopback_targets_allowed() {
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
