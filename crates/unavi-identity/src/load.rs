use std::{
    path::PathBuf,
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
    LocalBlobStore,
    LocalBlobs,
    LocalDocs,
    LocalDownloader,
    LocalGossip,
    SyncTargets,
    set_local_actor,
    set_root_doc,
};
use iroh::Endpoint;
use unavi_util::{
    async_commands::AsyncCommands,
    async_task::spawn_async_task,
};
use wds::{
    DataStore,
    identity::{
        RootIdentity,
        labels,
    },
};

use crate::{
    InMemory,
    LocalIdentity,
    SyncConfig,
    registry,
};

pub fn spawn_actors(
    trigger: On<Add, IrohEndpoint>,
    endpoints: Query<&IrohEndpoint>,
    identity: Res<LocalIdentity>,
    in_memory: Res<InMemory>,
    sync: Res<SyncConfig>,
) {
    let entity = trigger.entity;

    let endpoint = endpoints
        .get(entity)
        .map(|e| e.0.clone())
        .expect("endpoint");

    let identity = Arc::clone(&identity.0);
    let storage = store_path(in_memory.0);
    let sync = sync.clone();

    spawn_async_task(async move {
        let mut delay_secs = 4;

        loop {
            if let Err(err) = load_store(
                endpoint.clone(),
                Arc::clone(&identity),
                entity,
                storage.clone(),
                sync.clone(),
            )
            .await
            {
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

/// Wasm has no filesystem to back a store with, so it stays in memory and
/// refetches content each session.
#[cfg(target_family = "wasm")]
const fn store_path(_in_memory: bool) -> Option<PathBuf> {
    None
}

#[cfg(not(target_family = "wasm"))]
fn store_path(in_memory: bool) -> Option<PathBuf> {
    (!in_memory).then(|| unavi_util::dirs::data_local_dir().join("wds"))
}

async fn load_store(
    endpoint: Endpoint,
    identity: Arc<RootIdentity>,
    entity: Entity,
    storage: Option<PathBuf>,
    sync: SyncConfig,
) -> anyhow::Result<()> {
    let builder = DataStore::builder(endpoint.clone(), identity).gc_timer(Duration::from_mins(15));
    let builder = match storage {
        Some(path) => builder.storage_path(path),
        None => builder,
    };

    let (store, f) = builder.build().await?;
    let store = Arc::new(store);

    let actor = store.local_actor();
    set_local_actor(actor.clone());

    // Derived from the identity rather than minted, so the entries written here
    // last session are the entries read this one.
    let root = wds::docs::ensure_writable(store.docs(), store.namespace(labels::ROOT_DOC)).await?;
    set_root_doc(root.id());

    let SyncConfig {
        allow_loopback,
        targets,
    } = sync;

    let (sync_targets, unresolved) = registry::resolve_batch(&store, targets, allow_loopback).await;
    registry::sync(&store, &endpoint, &sync_targets).await;

    let store_entity = AsyncCommands::default()
        .spawn((RouterBuilderFnTarget(entity), RouterBuilderFn(Some(f))))
        .send_spawn((
            LocalActor(actor),
            LocalBlobStore(store.blobs().clone()),
            LocalBlobs(store.blobs().blobs().clone()),
            LocalDownloader(store.blobs().downloader(&endpoint)),
            LocalDocs(store.docs().clone()),
            LocalGossip(store.gossip().clone()),
            SyncTargets(sync_targets.clone()),
        ))
        .await;

    if !unresolved.is_empty() {
        spawn_async_task(registry::retry(
            store,
            endpoint,
            sync_targets,
            unresolved,
            store_entity,
            allow_loopback,
        ));
    }

    Ok(())
}
