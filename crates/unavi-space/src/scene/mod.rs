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
use bevy_wds::{
    LocalBlobs,
    LocalDocs,
    SyncTargets,
};
use loro::LoroDoc;
use tokio::sync::oneshot;
use unavi_util::async_task::spawn_async_task;
use wds::space;

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
    let peers = sync_targets
        .0
        .iter()
        .map(|a| a.host().clone())
        .collect::<Vec<_>>();

    let (tx, rx) = async_channel::bounded(1);
    let (cancel_tx, cancel_rx) = oneshot::channel();

    spawn_async_task(async move {
        let fetch = space::fetch_snapshot(&docs, &blobs, ns, peers, READ_ATTEMPTS, READ_DELAY);
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
