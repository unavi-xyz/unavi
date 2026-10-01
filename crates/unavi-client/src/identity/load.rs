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
    router,
    store::{
        DataStore,
        SyncTargets,
    },
};
use iroh::Endpoint;
use n0_future::task::AbortOnDropHandle;
use unavi_identity::{
    auth::{
        self,
        Bindings,
    },
    identity::{
        Identity,
        NodeIdentity,
        root_document,
    },
    resolver::Resolver,
};
use unavi_registry::client::{
    Followed,
    Target,
    resolve_batch,
};
use unavi_space::identity::{
    LocalIdentity,
    RootDocument,
};
use unavi_store::{
    Store,
    StoreBuilder,
};

use crate::identity::{
    Auth,
    KeyStorage,
    LocalNode,
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
                    router::accept(|builder| builder.accept(auth::ALPN, protocol)).apply(entity);
                }
            })
            .send()
            .await
            .ok();
    });
}

/// What following registries needs, carried into the load task.
#[derive(Clone)]
struct Follower {
    endpoint: Endpoint,
    bindings: Arc<Bindings>,
    identity: Arc<Identity>,
    resolver: Arc<Resolver>,
    followed: Followed,
}

impl Follower {
    async fn sync(&self, store: &Store, targets: &[Target]) {
        self.followed
            .sync(
                store,
                &self.endpoint,
                &self.bindings,
                targets,
                &self.identity,
                &self.resolver,
            )
            .await;
    }
}

pub fn load_store(
    trigger: On<Add, IrohEndpoint>,
    endpoints: Query<&IrohEndpoint>,
    node: Res<LocalNode>,
    local: Res<LocalIdentity>,
    storage: Res<KeyStorage>,
    sync: Res<SyncConfig>,
    async_world: Res<AsyncWorld>,
) {
    let entity = trigger.entity;

    let endpoint = endpoints
        .get(entity)
        .map(|e| e.0.clone())
        .expect("endpoint");

    let node = Arc::clone(&node.0);
    let follower = Follower {
        endpoint,
        bindings: Arc::clone(&local.bindings),
        identity: Arc::clone(node.user()),
        resolver: Arc::clone(&local.resolver),
        followed: local.followed.clone(),
    };
    let storage = storage.0.clone();
    let sync = sync.clone();
    let async_world = async_world.clone();

    task::spawn(async move {
        let mut delay_secs = 4;

        // The store shuts down with the last handle to it, which is the one
        // this hands to the endpoint entity.
        while let Err(err) = load(
            &async_world,
            &follower,
            &node,
            entity,
            storage.clone(),
            sync.clone(),
        )
        .await
        {
            error!(?err, "Failed to load data store");
            n0_future::time::sleep(Duration::from_secs(delay_secs)).await;
            delay_secs = delay_secs.saturating_mul(2).min(MAX_RETRY_DELAY.as_secs());
        }
    });
}

async fn load(
    async_world: &AsyncWorld,
    follower: &Follower,
    node: &NodeIdentity,
    entity: Entity,
    storage: unavi_local::DeviceStorage,
    sync: SyncConfig,
) -> anyhow::Result<()> {
    let builder = StoreBuilder::new(follower.endpoint.clone(), node.author())
        .sweep_interval(Duration::from_mins(15))
        .doc_budget(DOC_BUDGET)
        .storage(storage.clone());

    let store = builder.build().await?;

    let SyncConfig { targets } = sync;

    let (targets, unresolved) = resolve_batch(targets, &follower.resolver).await;
    follower.sync(&store, &targets).await;

    let root = root_document::open(&store, node.user()).await?.id();
    let sync_targets = targets
        .iter()
        .map(|target| target.addr.clone())
        .collect::<Vec<_>>();

    let installed = async_world
        .commands()
        .push({
            let store = store.clone();
            move |world: &mut World| {
                if let Ok(entity) = world.get_entity_mut(entity) {
                    let accepting = store.clone();
                    router::accept(move |builder| accepting.accept(builder)).apply(entity);
                }
                world.insert_resource(RootDocument(root));
                world.insert_resource(SyncTargets(sync_targets));
                world.insert_resource(DataStore(store));
            }
        })
        .send()
        .await;
    if installed.is_err() {
        warn!("the world is gone; dropping the data store");
        return Ok(());
    }

    if !unresolved.is_empty() {
        task::spawn(retry(
            async_world.clone(),
            store.clone(),
            follower.clone(),
            targets,
            unresolved,
        ));
    }

    Ok(())
}

/// Keeps resolving the registries that were unreachable at startup, so a server
/// brought up after the client is still followed without a restart.
async fn retry(
    async_world: AsyncWorld,
    store: Store,
    follower: Follower,
    mut targets: Vec<Target>,
    mut unresolved: Vec<String>,
) {
    let mut delay = RETRY_DELAY;

    while !unresolved.is_empty() {
        n0_future::time::sleep(delay).await;
        delay = (delay * 2).min(MAX_RETRY_DELAY);

        let (resolved, pending) = resolve_batch(unresolved, &follower.resolver).await;
        unresolved = pending;

        if resolved.is_empty() {
            continue;
        }
        targets.extend(resolved);

        follower.sync(&store, &targets).await;

        let published = targets
            .iter()
            .map(|target| target.addr.clone())
            .collect::<Vec<_>>();
        let sent = async_world
            .commands()
            .push(move |world: &mut World| {
                world.insert_resource(SyncTargets(published));
            })
            .send()
            .await;

        if let Err(err) = sent {
            error!(?err, "failed to publish resolved sync targets");
            return;
        }
    }
}
