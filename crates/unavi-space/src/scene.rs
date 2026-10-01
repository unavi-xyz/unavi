//! Fetching a space's own document and instancing it on the space entity.

use std::time::Duration;

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
use bevy_iroh::store::{
    DataStore,
    SyncTargets,
};
use hsd::state::HsdState;
use iroh::EndpointAddr;
use iroh_docs::NamespaceId;
use tokio::sync::oneshot;
use unavi_store::Document;

use crate::{
    discovery::{
        LastSeenIn,
        Peer,
    },
    membership::Space,
};

/// How long [`start_space_fetch`] gives gossip to confirm an occupant before
/// reading the space with whatever it found. A registry-listed occupant
/// gossip has not reached would only fail the exact dial iroh-docs' own sync
/// engine already tried, for a warning with nothing new to say.
const PEER_WAIT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Component)]
pub struct PendingScene {
    /// Carries the document alongside its feed, so the component that lands
    /// on the entity holds the same handle the fetch opened.
    rx:      Receiver<(Document, DocFeed)>,
    peers:   Vec<EndpointAddr>,
    _cancel: oneshot::Sender<()>,
}

/// A space entered but not yet read, waiting on [`start_space_fetch`] for
/// gossip to confirm someone worth asking.
#[derive(Component)]
pub struct PendingSpacePeers {
    ns:            NamespaceId,
    sync_targets:  Vec<EndpointAddr>,
    waiting_since: Duration,
}

pub fn spawn_space_scene(
    trigger: On<Add, Space>,
    spaces: Query<(&Space, Option<&HsdNamespace>)>,
    store: Option<Res<DataStore>>,
    sync_targets: Option<Res<SyncTargets>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let Ok((ns, instanced)) = spaces
        .get(trigger.entity)
        .map(|(space, ns)| (space.namespace(), ns.map(|doc| doc.0.id())))
    else {
        return;
    };

    let Some(store) = store else {
        warn!("Cannot read space: no local store");
        return;
    };

    // A locally built space is already in the scene on this entity; reading
    // it back would duplicate every prim and re-run every script. It still
    // has to be served: presence is announced for it, so peers arrive
    // expecting an answer.
    if instanced == Some(ns) {
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
        return;
    }
    info!(%ns, "Reading space");

    commands
        .entity(trigger.entity)
        .try_insert(PendingSpacePeers {
            ns,
            sync_targets: sync_targets.map_or_default(|targets| targets.0.clone()),
            waiting_since: time.elapsed(),
        });
}

/// Starts the actual space-document fetch once gossip has confirmed an
/// occupant for `ns` (see [`LastSeenIn`]), or the wait times
/// out. A `NeighborUp` on that space's gossip topic fast-tracks a presence
/// exchange (see [`crate::discovery::gossip`]), so a real
/// occupant shows up here within milliseconds of connecting, not the full
/// presence broadcast interval.
///
/// A configured sync target never skips this wait on its own: a space's
/// content lives with its occupants, and a general-purpose sync target
/// answers "not found" for a space it was never asked to hold — firing on it
/// alone raced out the occupant that was a tick away from confirming.
pub fn start_space_fetch(
    time: Res<Time>,
    pending: Query<(Entity, &PendingSpacePeers)>,
    store: Option<Res<DataStore>>,
    active_peers: Query<(&Peer, &LastSeenIn)>,
    mut commands: Commands,
) {
    let Some(store) = store else {
        return;
    };

    for (entity, waiting) in &pending {
        let confirmed = active_peers
            .iter()
            .filter(|(_, seen)| seen.contains(waiting.ns.into()))
            .map(|(peer, _)| peer.0.clone())
            .collect::<Vec<_>>();

        let timed_out = time.elapsed().saturating_sub(waiting.waiting_since) >= PEER_WAIT_TIMEOUT;
        if confirmed.is_empty() && !timed_out {
            continue;
        }

        let mut peers = waiting.sync_targets.clone();
        peers.extend(confirmed);

        let ns = waiting.ns;
        let store = store.0.clone();
        let (tx, rx) = async_channel::bounded(1);
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let sync_peers = peers.clone();

        task::spawn(async move {
            let fetch = async {
                let (doc, events) = store.join(ns, peers).await?;
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

        commands
            .entity(entity)
            .try_remove::<PendingSpacePeers>()
            .try_insert(PendingScene {
                rx,
                peers: sync_peers,
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
            ))
            .try_remove::<PendingScene>();
    }
}

pub fn despawn_space_scene(trigger: On<Remove, Space>, mut commands: Commands) {
    // Removing PendingScene drops the oneshot::Sender, signalling the task to
    // cancel. Despawning the space drops it just as well, so a missing entity
    // is not an error.
    commands
        .entity(trigger.entity)
        .try_remove::<(PendingScene, Hsd)>();
}
