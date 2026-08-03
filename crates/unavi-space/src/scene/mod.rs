use std::{
    sync::Arc,
    time::Duration,
};

use async_channel::Receiver;
use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdNamespace,
};
use bevy_iroh::endpoint::IrohEndpoint;
use bevy_wds::{
    LocalBlobs,
    LocalDocs,
    SyncTargets,
    registry_clients,
};
use iroh::{
    EndpointAddr,
    EndpointId,
};
use iroh_docs::NamespaceId;
use loro::LoroDoc;
use tokio::sync::oneshot;
use unavi_util::async_task::spawn_async_task;
use wds::snapshot;

use crate::Space;

pub mod pinned_docs;

const READ_ATTEMPTS: usize = 10;
const READ_DELAY: Duration = Duration::from_secs(1);

#[derive(Component)]
pub struct PendingScene {
    rx:      Receiver<LoroDoc>,
    _cancel: oneshot::Sender<()>,
}

pub fn spawn_space_scene(
    trigger: On<Add, Space>,
    spaces: Query<&Space>,
    stores: Query<(&LocalDocs, &LocalBlobs, &SyncTargets)>,
    endpoints: Query<&IrohEndpoint>,
    mut commands: Commands,
) {
    let ns = spaces.get(trigger.entity).map(|v| v.0).expect("space");
    let Ok((docs, blobs, sync_targets)) = stores.single() else {
        warn!("Cannot read space: no local store");
        return;
    };
    info!(%ns, "Reading space");

    let docs = docs.0.clone();
    let blobs = blobs.0.clone();
    let self_id = endpoints.single().ok().map(|e| e.0.id());
    let sync_targets = sync_targets
        .0
        .iter()
        .map(|a| a.host().clone())
        .collect::<Vec<_>>();

    let (tx, rx) = async_channel::bounded(1);
    let (cancel_tx, cancel_rx) = oneshot::channel();

    spawn_async_task(async move {
        let mut peers = sync_targets;
        peers.extend(occupant_peers(ns, self_id).await);

        let fetch = snapshot::fetch(&docs, &blobs, ns, peers, READ_ATTEMPTS, READ_DELAY);
        tokio::select! {
            () = async { cancel_rx.await.ok(); } => {}
            res = fetch => match res {
                Ok(Some(bytes)) => {
                    let doc = LoroDoc::new();
                    if doc.import(&bytes).is_ok() {
                        tx.send(doc).await.ok();
                    }
                }
                Ok(None) => warn!(%ns, "space snapshot never arrived"),
                Err(err) => error!(?err, "failed reading space snapshot"),
            },
        }
    });

    commands.entity(trigger.entity).insert(PendingScene {
        rx,
        _cancel: cancel_tx,
    });
}

/// Endpoints currently in `ns`, per the registries this client follows.
///
/// A space's document lives with the peers who are in it. A home server holds a
/// copy only if it was explicitly asked to host one, so syncing against sync
/// targets alone finds nothing for a peer-hosted space.
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
        let Ok(doc) = pending.rx.try_recv() else {
            continue;
        };

        info!(space = %space.0, "Instantiating scene");
        commands
            .entity(entity)
            .insert((Hsd(Arc::new(doc)), HsdNamespace(space.0)))
            .remove::<PendingScene>();
    }
}

pub fn despawn_space_scene(trigger: On<Remove, Space>, mut commands: Commands) {
    // Removing PendingScene drops the oneshot::Sender, signalling the task to
    // cancel.
    commands
        .entity(trigger.entity)
        .remove::<(PendingScene, Hsd)>();
}
