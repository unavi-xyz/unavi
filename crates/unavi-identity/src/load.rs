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
    SyncTargets,
    set_local_actor,
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

    let (store, f) = DataStore::builder(endpoint)
        .gc_timer(Duration::from_mins(15))
        .build()
        .await?;

    store.set_user_identity(Arc::clone(&identity));
    let actor = store.local_actor(Arc::clone(&identity));
    set_local_actor(actor.clone());

    let sync_targets = load_sync_targets(&store, &identity).await;

    AsyncCommands::default()
        .spawn((RouterBuilderFnTarget(entity), RouterBuilderFn(Some(f))))
        .spawn((
            LocalActor(actor),
            LocalBlobs(store.blobs().blobs().clone()),
            SyncTargets(sync_targets),
        ))
        .send()
        .await?;

    Ok(())
}

async fn load_sync_targets(store: &DataStore, identity: &Arc<Identity>) -> Vec<Actor> {
    let Ok(raw) = std::env::var("UNAVI_SYNC_TARGETS") else {
        return Vec::new();
    };

    let dids: Vec<&str> = raw
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    if dids.is_empty() {
        return Vec::new();
    }

    let resolver = match DidResolver::new() {
        Ok(r) => r,
        Err(err) => {
            error!(?err, "failed to construct DID resolver for sync targets");
            return Vec::new();
        }
    };

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

    actors
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
