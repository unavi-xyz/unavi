use std::{
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
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
use unavi_util::{
    async_commands::AsyncCommands,
    async_task::spawn_async_task,
};
use wds::{
    DataStore,
    Identity,
    WDS_SERVICE_TYPE,
    actor::Actor,
};
use wired_registry::client::RegistryClient;
use xdid::{
    core::did::Did,
    methods::key::{
        DidKeyPair,
        PublicKey,
        p256::P256KeyPair,
    },
    resolver::DidResolver,
};

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

    store.set_user_identity(Arc::clone(&identity));
    let actor = store.local_actor(Arc::clone(&identity));
    set_local_actor(actor.clone());

    if let Ok(root) = wds::kv::create(store.docs()).await {
        set_root_doc(root);
    }

    let sync_targets = load_sync_targets(&store, &identity).await?;
    sync_registries(&store, &endpoint, &sync_targets).await;

    AsyncCommands::default()
        .spawn((RouterBuilderFnTarget(entity), RouterBuilderFn(Some(f))))
        .spawn((
            LocalActor(actor),
            LocalBlobs(store.blobs().blobs().clone()),
            LocalDocs(store.docs().clone()),
            SyncTargets(sync_targets),
        ))
        .send()
        .await?;

    Ok(())
}

/// Builds a client per followed registry, syncs each one's view docs locally,
/// and publishes both for off-world access.
///
/// A registry is a service the client consults, never one it runs; with no sync
/// targets configured there is simply nothing to follow.
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

/// Resolves every configured sync target, or fails if none could be reached.
///
/// Failing is the point: a configured target that does not resolve is almost
/// always a server that has not finished starting, and the caller retries with
/// backoff. Returning an empty list instead would leave the client permanently
/// without a registry — no presence published, nothing discovered — with only a
/// warning to show for it.
async fn load_sync_targets(
    store: &DataStore,
    identity: &Arc<Identity>,
) -> anyhow::Result<Vec<Actor>> {
    let Ok(raw) = std::env::var("UNAVI_SYNC_TARGETS") else {
        return Ok(Vec::new());
    };

    let dids: Vec<&str> = raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    if dids.is_empty() {
        return Ok(Vec::new());
    }

    let resolver = DidResolver::new().context("construct DID resolver for sync targets")?;
    let mut actors = Vec::new();

    for did_str in dids {
        match resolve_sync_target(&resolver, did_str).await {
            Ok(addr) => {
                info!(target = did_str, "registering WDS sync target");
                actors.push(store.remote_actor(Arc::clone(identity), addr));
            }
            Err(err) => {
                warn!(target = did_str, ?err, "failed to resolve WDS sync target");
            }
        }
    }

    if actors.is_empty() {
        anyhow::bail!("no configured sync target resolved: {raw}");
    }

    Ok(actors)
}

async fn resolve_sync_target(
    resolver: &DidResolver,
    did_str: &str,
) -> anyhow::Result<EndpointAddr> {
    let did = Did::from_str(did_str)?;
    let doc = resolver.resolve(&did).await?;

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
