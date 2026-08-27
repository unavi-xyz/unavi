use std::{
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
    store::{
        LocalBlobStore,
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
    },
    resolve::Resolver,
};
use unavi_registry::follow;
use unavi_store::{
    local,
    store::{
        Builder as StoreBuilder,
        Guard,
        Spawned,
        Store,
    },
};
use unavi_util::{
    async_commands::AsyncCommands,
    async_task::spawn_async_task,
};

use crate::identity::{
    Auth,
    LocalNode,
    Resolve,
    Storage,
    SyncConfig,
};

const RETRY_DELAY: Duration = Duration::from_secs(4);
const MAX_RETRY_DELAY: Duration = Duration::from_mins(1);

/// Holds the `wired/auth` outgoing-handshake task for as long as the endpoint
/// entity lives.
#[derive(Component)]
struct AuthTask(#[expect(dead_code, reason = "aborts the task on drop")] AbortOnDropHandle<()>);

/// Answers `wired/auth` on the endpoint the hooks were installed on.
pub fn serve_auth(
    trigger: On<Add, IrohEndpoint>,
    endpoints: Query<&IrohEndpoint>,
    auth: Res<Auth>,
    mut commands: Commands,
) {
    let entity = trigger.entity;
    let Ok(endpoint) = endpoints.get(entity).map(|e| e.0.clone()) else {
        return;
    };

    let Some((protocol, task)) = auth.0.serve(endpoint) else {
        warn!("a second endpoint cannot serve the same identity handshake");
        return;
    };

    commands.entity(entity).insert(AuthTask(task));
    commands.spawn((
        RouterBuilderFnTarget(entity),
        RouterBuilderFn(Some(Box::new(|builder| {
            builder.accept(auth::ALPN, protocol)
        }))),
    ));
}

pub fn load_store(
    trigger: On<Add, IrohEndpoint>,
    endpoints: Query<&IrohEndpoint>,
    node: Res<LocalNode>,
    storage: Res<Storage>,
    sync: Res<SyncConfig>,
    resolve: Res<Resolve>,
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

    spawn_async_task(async move {
        let mut delay_secs = 4;

        let _guard = loop {
            match load(
                endpoint.clone(),
                Arc::clone(&node),
                entity,
                storage.clone(),
                sync.clone(),
                Arc::clone(&resolver),
            )
            .await
            {
                Ok(guard) => break guard,
                Err(err) => {
                    error!(?err, "Failed to load data store");
                    n0_future::time::sleep(Duration::from_secs(delay_secs)).await;
                    delay_secs = delay_secs.wrapping_mul(2);
                }
            }
        };

        // Dropping the guard shuts the blob store down, so it is held for as
        // long as the process runs.
        std::future::pending::<()>().await;
    });
}

async fn load(
    endpoint: Endpoint,
    node: Arc<NodeIdentity>,
    entity: Entity,
    storage: local::Storage,
    sync: SyncConfig,
    resolver: Arc<Resolver>,
) -> anyhow::Result<Guard> {
    let builder = StoreBuilder::new(endpoint.clone(), node.author())
        .gc_timer(Duration::from_mins(15))
        .storage(storage.clone());

    let Spawned {
        store,
        router,
        guard,
    } = builder.build().await?;

    let SyncConfig {
        allow_loopback,
        targets,
    } = sync;

    let identity = Arc::clone(node.user());
    let (sync_targets, unresolved) =
        follow::resolve_batch(targets, allow_loopback, &resolver).await;
    follow::sync(
        &store,
        &endpoint,
        &sync_targets,
        Arc::clone(&identity),
        Arc::clone(&resolver),
    )
    .await;

    let store_entity = AsyncCommands::default()
        .spawn((RouterBuilderFnTarget(entity), RouterBuilderFn(Some(router))))
        .send_spawn((
            LocalBlobStore(store.blob_store().clone()),
            LocalBlobs(store.blobs().clone()),
            LocalDownloader(store.blob_store().downloader(&endpoint)),
            LocalStore(store.clone()),
            SyncTargets(sync_targets.clone()),
        ))
        .await;

    if !unresolved.is_empty() {
        spawn_async_task(retry(
            store.clone(),
            endpoint,
            sync_targets,
            unresolved,
            store_entity,
            allow_loopback,
            identity,
            resolver,
        ));
    }

    Ok(guard)
}

/// Keeps resolving the registries that were unreachable at startup, so a server
/// brought up after the client is still followed without a restart.
async fn retry(
    store: Store,
    endpoint: Endpoint,
    mut targets: Vec<EndpointAddr>,
    mut unresolved: Vec<String>,
    store_entity: Entity,
    allow_loopback: bool,
    identity: Arc<Identity>,
    resolver: Arc<Resolver>,
) {
    let mut delay = RETRY_DELAY;

    while !unresolved.is_empty() {
        n0_future::time::sleep(delay).await;
        delay = (delay * 2).min(MAX_RETRY_DELAY);

        let (addrs, pending) = follow::resolve_batch(unresolved, allow_loopback, &resolver).await;
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
