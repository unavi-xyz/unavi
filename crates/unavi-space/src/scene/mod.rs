use std::time::Duration;

use async_channel::Receiver;
use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdDocId,
    HsdNamespace,
    document,
};
use bevy_iroh::{
    endpoint::IrohEndpoint,
    store::{
        LocalStore,
        SyncTargets,
    },
};
use hsd::{
    id::DocId,
    key,
    state::SceneState,
};
use iroh::{
    EndpointAddr,
    EndpointId,
};
use iroh_docs::NamespaceId;
use tokio::sync::oneshot;
use unavi_policy::space::Space;
use unavi_registry::follow::registry_clients;
use unavi_util::async_task::spawn_async_task;

pub mod pinned_docs;

const READ_ATTEMPTS: usize = 10;
const READ_DELAY: Duration = Duration::from_secs(1);

#[derive(Component)]
pub struct PendingScene {
    rx:      Receiver<SceneState>,
    _cancel: oneshot::Sender<()>,
}

pub fn spawn_space_scene(
    trigger: On<Add, Space>,
    spaces: Query<(&Space, Option<&HsdNamespace>)>,
    stores: Query<(&LocalStore, &SyncTargets)>,
    endpoints: Query<&IrohEndpoint>,
    mut commands: Commands,
) {
    let (ns, instanced) = spaces
        .get(trigger.entity)
        .map(|(space, ns)| (space.0, ns.map(|v| v.0)))
        .expect("space");

    let Ok((store, sync_targets)) = stores.single() else {
        warn!("Cannot read space: no local store");
        return;
    };

    // A locally built space is already realized on this entity; reading it back
    // would duplicate every prim and re-run every script. It still has to be
    // served: presence is announced for it, so peers arrive expecting an answer.
    if instanced == Some(ns) {
        let store = store.0.clone();
        spawn_async_task(async move {
            let served = async { store.open(ns).await?.serve().await };
            if let Err(err) = served.await {
                warn!(%ns, ?err, "Failed to serve local space");
            }
        });
        return;
    }
    info!(%ns, "Reading space");

    let store = store.0.clone();
    let self_id = endpoints.single().ok().map(|e| e.0.id());
    let sync_targets = sync_targets.0.clone();

    let (tx, rx) = async_channel::bounded(1);
    let (cancel_tx, cancel_rx) = oneshot::channel();

    spawn_async_task(async move {
        let mut peers = sync_targets;
        peers.extend(occupant_peers(ns, self_id).await);

        // Waiting on the prim prefix rather than a single snapshot key: the
        // document is its entries now, and the first prim proves it arrived.
        let fetch = async {
            let doc = store.open(ns).await?;
            doc.sync_from(peers).await?;
            let arrived = doc
                .wait_for(key::PRIM_PREFIX, READ_ATTEMPTS, READ_DELAY)
                .await?;
            anyhow::Ok((doc, arrived))
        };
        tokio::select! {
            () = async { cancel_rx.await.ok(); } => {}
            res = fetch => match res {
                Ok((doc, true)) => match document::read_state(&doc).await {
                    Ok(state) => {
                        tx.send(state).await.ok();
                    }
                    Err(err) => error!(?err, "failed reading space entries"),
                },
                Ok((_, false)) => warn!(%ns, "space document never arrived"),
                Err(err) => error!(?err, "failed syncing space document"),
            },
        }
    });

    commands.entity(trigger.entity).insert(PendingScene {
        rx,
        _cancel: cancel_tx,
    });
}

/// Endpoints currently in `ns`, per the registries this client follows. A
/// space's content lives with its occupants; a home server holds none unless
/// asked, so sync targets alone find nothing for a peer-hosted space.
async fn occupant_peers(ns: NamespaceId, self_id: Option<EndpointId>) -> Vec<EndpointAddr> {
    let mut out = Vec::new();

    for registry in registry_clients() {
        let Ok(occupants) = registry.occupants(ns).await else {
            continue;
        };
        for presence in occupants {
            let Ok(id) = EndpointId::from_bytes(&presence.endpoint) else {
                continue;
            };
            if Some(id) == self_id {
                continue;
            }
            out.push(EndpointAddr::from(id));
        }
    }

    out
}

pub fn instantiate_pending_scenes(
    pending: Query<(Entity, &Space, &PendingScene)>,
    mut commands: Commands,
) {
    for (entity, space, pending) in &pending {
        let Ok(state) = pending.rx.try_recv() else {
            continue;
        };

        info!(space = %space.0, "Instantiating scene");
        commands
            .entity(entity)
            .insert((
                Hsd::new(state),
                HsdDocId(DocId(*space.0.as_bytes())),
                HsdNamespace(space.0),
            ))
            .remove::<PendingScene>();
    }
}

pub fn despawn_space_scene(trigger: On<Remove, Space>, mut commands: Commands) {
    // Removing PendingScene drops the oneshot::Sender, signalling the task to
    // cancel. Despawning the space drops it just as well, so a missing entity is
    // not an error.
    commands
        .entity(trigger.entity)
        .try_remove::<(PendingScene, Hsd)>();
}
