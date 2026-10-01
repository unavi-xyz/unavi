use std::{
    sync::Arc,
    time::Duration,
};

use bevy::prelude::*;
use bevy_async::{
    AsyncWorld,
    task,
};
use bevy_iroh::{
    endpoint::IrohEndpoint,
    router::{
        RouterBuilderFn,
        RouterBuilderFnTarget,
    },
    store::{
        LocalBlobs,
        LocalDownloader,
        LocalStore,
        SyncTargets,
    },
};
use iroh::{
    Endpoint,
    EndpointAddr,
};
use n0_future::task::AbortOnDropHandle;
use unavi_identity::{
    auth,
    identity::{
        Identity,
        NodeIdentity,
        root,
    },
};
use unavi_registry::follow;
use unavi_space::identity::RootDocument;
use unavi_store::{
    Store,
    builder::StoreBuilder,
};
use xdid::resolver::DidResolver;

use crate::identity::{
    Auth,
    LocalNode,
    LocalStorage,
    Resolve,
    SyncConfig,
};

const RETRY_DELAY: Duration = Duration::from_secs(4);
const MAX_RETRY_DELAY: Duration = Duration::from_mins(1);

const GB: u64 = 1024 * 1024 * 1024;

/// How many bytes of cached documents this device keeps before the
/// oldest-visited ones are evicted.
///
/// Web holds everything in memory and loses it on reload anyway, so its number
/// is a ceiling on one session rather than on a disk. Neither figure has been
/// measured against a real cache hit rate.
#[cfg(not(target_family = "wasm"))]
const DOC_BUDGET: u64 = 8 * GB;
#[cfg(target_family = "wasm")]
const DOC_BUDGET: u64 = GB / 4;

/// Holds the `wired/auth` outgoing-handshake task for as long as the endpoint
/// entity lives.
#[derive(Component)]
struct AuthTask(#[expect(dead_code, reason = "aborts the task on drop")] AbortOnDropHandle<()>);

/// Answers `wired/auth` on the endpoint the hooks were installed on.
pub fn serve_auth(
    trigger: On<Add, IrohEndpoint>,
    endpoints: Query<&IrohEndpoint>,
    auth: Res<Auth>,
    async_world: Res<AsyncWorld>,
) {
    let entity = trigger.entity;
    let Ok(endpoint) = endpoints.get(entity).map(|e| e.0.clone()) else {
        return;
    };
    let auth = Arc::clone(&auth.0);
    let async_world = async_world.clone();

    // `EndpointAuth::serve` spawns a background task with `tokio::spawn`,
    // which needs to run inside the tokio runtime, not on this observer's
    // calling thread.
    task::spawn(async move {
        let Some((protocol, task)) = auth.serve(endpoint) else {
            warn!("a second endpoint cannot serve the same identity handshake");
            return;
        };

        async_world
            .commands()
            .push(move |world: &mut World| {
                if let Ok(mut entity) = world.get_entity_mut(entity) {
                    entity.insert(AuthTask(task));
                }
            })
            .spawn((
                RouterBuilderFnTarget(entity),
                RouterBuilderFn(Some(Box::new(|builder| {
                    builder.accept(auth::ALPN, protocol)
                }))),
            ))
            .send()
            .await
            .ok();
    });
}

pub fn load_store(
    trigger: On<Add, IrohEndpoint>,
    endpoints: Query<&IrohEndpoint>,
    node: Res<LocalNode>,
    storage: Res<LocalStorage>,
    sync: Res<SyncConfig>,
    resolve: Res<Resolve>,
    async_world: Res<AsyncWorld>,
) {
    let entity = trigger.entity;

    let endpoint = endpoints
        .get(entity)
        .map(|e| e.0.clone())
        .expect("endpoint");

    let node = Arc::clone(&node.0);
    let storage = storage.0.clone();
    let sync = sync.clone();
    let resolver = Arc::clone(&resolve.0);
    let async_world = async_world.clone();

    task::spawn(async move {
        let mut delay_secs = 4;

        // The store shuts down with the last handle to it, which is the one
        // this hands to the endpoint entity.
        while let Err(err) = load(
            &async_world,
            endpoint.clone(),
            Arc::clone(&node),
            entity,
            storage.clone(),
            sync.clone(),
            Arc::clone(&resolver),
        )
        .await
        {
            error!(?err, "Failed to load data store");
            n0_future::time::sleep(Duration::from_secs(delay_secs)).await;
            delay_secs = delay_secs.wrapping_mul(2);
        }
    });
}

async fn load(
    async_world: &AsyncWorld,
    endpoint: Endpoint,
    node: Arc<NodeIdentity>,
    entity: Entity,
    storage: unavi_local::LocalStorage,
    sync: SyncConfig,
    resolver: Arc<DidResolver>,
) -> anyhow::Result<()> {
    let builder = StoreBuilder::new(endpoint.clone(), node.author())
        .gc_timer(Duration::from_mins(15))
        .doc_budget(DOC_BUDGET)
        .storage(storage.clone());

    let store = builder.build().await?;

    let SyncConfig { targets } = sync;

    let identity = Arc::clone(node.user());
    let (sync_targets, unresolved) = follow::resolve_batch(targets, &resolver).await;
    follow::sync(
        &store,
        &endpoint,
        &sync_targets,
        Arc::clone(&identity),
        Arc::clone(&resolver),
    )
    .await;

    let root = root::open(&store).await?.id();

    let Some(store_entity) = async_world
        .commands()
        .spawn((
            RouterBuilderFnTarget(entity),
            RouterBuilderFn(Some(Box::new({
                let store = store.clone();
                move |builder| store.accept(builder)
            }))),
        ))
        .push(move |world: &mut World| {
            world.insert_resource(RootDocument(root));
        })
        .send_spawn((
            LocalBlobs(store.blobs().clone()),
            LocalDownloader(store.blob_store().downloader(&endpoint)),
            LocalStore(store.clone()),
            SyncTargets(sync_targets.clone()),
        ))
        .await
    else {
        warn!("the world is gone; dropping the data store");
        return Ok(());
    };

    if !unresolved.is_empty() {
        task::spawn(retry(
            async_world.clone(),
            store.clone(),
            endpoint,
            sync_targets,
            unresolved,
            store_entity,
            identity,
            resolver,
        ));
    }

    Ok(())
}

/// Keeps resolving the registries that were unreachable at startup, so a server
/// brought up after the client is still followed without a restart.
async fn retry(
    async_world: AsyncWorld,
    store: Store,
    endpoint: Endpoint,
    mut targets: Vec<EndpointAddr>,
    mut unresolved: Vec<String>,
    store_entity: Entity,
    identity: Arc<Identity>,
    resolver: Arc<DidResolver>,
) {
    let mut delay = RETRY_DELAY;

    while !unresolved.is_empty() {
        n0_future::time::sleep(delay).await;
        delay = (delay * 2).min(MAX_RETRY_DELAY);

        let (addrs, pending) = follow::resolve_batch(unresolved, &resolver).await;
        unresolved = pending;

        if addrs.is_empty() {
            continue;
        }
        targets.extend(addrs);

        follow::sync(
            &store,
            &endpoint,
            &targets,
            Arc::clone(&identity),
            Arc::clone(&resolver),
        )
        .await;

        let published = targets.clone();
        let sent = async_world
            .commands()
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
