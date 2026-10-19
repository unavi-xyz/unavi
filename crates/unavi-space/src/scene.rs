//! Fetching a space's own document and instancing it on the space entity.

use async_channel::Receiver;
use bevy::prelude::*;
use bevy_async::task;
use bevy_hsd::{
    document::{
        Hsd,
        HsdDocId,
        HsdNamespace,
        SyncPeers,
    },
    feed::{
        DocFeed,
        FeedReady,
    },
};
use bevy_iroh::store::DataStore;
use hsd::state::HsdState;
use iroh::EndpointAddr;
use tokio::sync::oneshot;
use unavi_store::Document;

use crate::{
    discovery::{
        LastSeenIn,
        Peer,
    },
    membership::{
        Space,
        SpaceId,
    },
    sync_peers,
};

#[derive(Component)]
pub struct PendingScene {
    /// Carries the document alongside its feed, so the component that lands
    /// on the entity holds the same handle the fetch opened.
    rx:      Receiver<(Document, DocFeed)>,
    peers:   Vec<EndpointAddr>,
    _cancel: oneshot::Sender<()>,
}

/// A space whose document is instanced on its entity or served locally.
#[derive(Component)]
pub struct SceneStarted;

/// Occupants of `space` that gossip has confirmed.
fn occupants<'a>(
    peers: &'a Query<(&Peer, &LastSeenIn)>,
    space: SpaceId,
) -> impl Iterator<Item = EndpointAddr> + 'a {
    peers
        .iter()
        .filter(move |(_, seen)| seen.contains(space))
        .map(|(peer, _)| peer.0.clone())
}

/// Starts reading every space that has not started yet, with whichever
/// occupants are known. Later occupants are named by [`name_late_occupants`].
///
/// Waits for the store, so a space added before it exists starts once it does.
pub fn start_space_scenes(
    spaces: Query<
        (Entity, &Space, Option<&HsdNamespace>),
        (Without<SceneStarted>, Without<PendingScene>),
    >,
    store: Option<Res<DataStore>>,
    peers: Query<(&Peer, &LastSeenIn)>,
    mut commands: Commands,
) {
    let Some(store) = store else {
        return;
    };

    for (entity, space, instanced) in &spaces {
        let ns = space.namespace();

        // A locally built space is already in the scene on this entity;
        // reading it back would duplicate every prim and re-run every script.
        // It still has to be served: presence is announced for it, so peers
        // arrive expecting an answer.
        if instanced.is_some_and(|doc| doc.0.id() == ns) {
            let store = store.0.clone();
            task::spawn(async move {
                let served = async {
                    let doc = store
                        .held(ns)
                        .await?
                        .ok_or(unavi_store::Error::NotHeld(ns))?;
                    doc.serve().await
                };
                if let Err(err) = served.await {
                    warn!(%ns, ?err, "Failed to serve local space");
                }
            });
            commands.entity(entity).try_insert(SceneStarted);
            continue;
        }
        info!(%ns, "Reading space");

        let known = occupants(&peers, space.id()).collect::<Vec<_>>();
        let named = known.clone();
        let store = store.0.clone();
        let (tx, rx) = async_channel::bounded(1);
        let (cancel_tx, cancel_rx) = oneshot::channel();

        task::spawn(async move {
            let fetch = async {
                let (doc, events) = store.join(ns, known).await?;
                let feed = DocFeed::joined(doc.clone(), events, FeedReady::RemoteSync);
                Ok::<_, unavi_store::Error>((doc, feed))
            };
            tokio::select! {
                () = async { cancel_rx.await.ok(); } => {}
                res = fetch => match res {
                    Ok(fetched) => {
                        if tx.send(fetched).await.is_err() {
                            debug!(%ns, "space fetched after its scene was dropped");
                        }
                    }
                    Err(err) => error!(?err, "failed syncing space document"),
                },
            }
        });

        commands.entity(entity).try_insert(PendingScene {
            rx,
            peers: named,
            _cancel: cancel_tx,
        });
    }
}

pub fn instance_pending_scenes(
    pending: Query<(Entity, &Space, &PendingScene)>,
    mut commands: Commands,
) {
    for (entity, space, pending) in &pending {
        let Ok((doc, feed)) = pending.rx.try_recv() else {
            continue;
        };

        info!(space = %space.id(), "Instancing space");
        commands
            .entity(entity)
            .try_insert((
                Hsd::new(HsdState::new()),
                HsdDocId(space.doc_id()),
                HsdNamespace(doc),
                SyncPeers(pending.peers.clone()),
                feed,
                SceneStarted,
            ))
            .try_remove::<PendingScene>();
    }
}

/// Names occupants confirmed after a space joined, whether or not its feed has
/// synced. Each occupant is named once.
pub fn name_late_occupants(
    mut spaces: Query<(&Space, &mut SyncPeers)>,
    peers: Query<(&Peer, &LastSeenIn)>,
    store: Option<Res<DataStore>>,
) {
    let Some(store) = store else {
        return;
    };

    for (space, mut named) in &mut spaces {
        let fresh = sync_peers::unnamed(&named, occupants(&peers, space.id()));
        if fresh.is_empty() {
            continue;
        }
        sync_peers::name_peers(&store.0, space.namespace(), fresh.clone());
        named.0.extend(fresh);
    }
}

pub fn despawn_space_scene(trigger: On<Remove, Space>, mut commands: Commands) {
    // Removing PendingScene drops the oneshot::Sender, signalling the task to
    // cancel. Despawning the space drops it just as well, so a missing entity
    // is not an error.
    commands
        .entity(trigger.entity)
        .try_remove::<(PendingScene, Hsd, SceneStarted)>();
}

#[cfg(test)] mod tests;
