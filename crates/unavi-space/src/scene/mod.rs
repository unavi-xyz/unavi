use std::time::Duration;

use async_channel::Receiver;
use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdDocId,
    HsdNamespace,
    document,
};
use bevy_iroh::store::{
    LocalStore,
    SyncTargets,
};
use hsd::{
    id::DocId,
    key,
    state::HsdState,
};
use iroh::EndpointAddr;
use iroh_docs::NamespaceId;
use tokio::sync::oneshot;
use unavi_policy::space::Space;
use unavi_util::async_task::spawn_async_task;
use wds::document::Document;

use crate::peer::{
    ActiveSpaces,
    Peer,
};

pub mod pinned_docs;

const READ_ATTEMPTS: usize = 10;
const READ_DELAY: Duration = Duration::from_secs(1);

/// Polls `doc` until an entry exists under `prefix`, reporting whether one
/// arrived before the attempts ran out.
///
/// A document syncs entry by entry, so the first entry of the expected shape is
/// the only signal that it has started to arrive.
async fn wait_for_prefix(
    doc: &Document,
    prefix: &str,
    attempts: usize,
    delay: Duration,
) -> anyhow::Result<bool> {
    for _ in 0..attempts.max(1) {
        let query = iroh_docs::store::Query::single_latest_per_key().key_prefix(prefix);
        if doc.get_one(query).await?.is_some() {
            return Ok(true);
        }
        n0_future::time::sleep(delay).await;
    }
    Ok(false)
}

/// How long [`start_space_fetch`] gives gossip to confirm an occupant before
/// reading the space with whatever it found. A registry-listed occupant
/// gossip has not reached would only fail the exact dial iroh-docs' own sync
/// engine already tried, for a warning with nothing new to say.
const PEER_WAIT_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Component)]
pub struct PendingScene {
    /// Carries the document alongside its state, so the component that lands
    /// on the entity holds the same handle the fetch opened.
    rx:      Receiver<(Document, HsdState)>,
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
    stores: Query<(&LocalStore, &SyncTargets)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let (ns, instanced) = spaces
        .get(trigger.entity)
        .map(|(space, ns)| (space.0, ns.map(|doc| doc.0.id())))
        .expect("space");

    let Ok((store, sync_targets)) = stores.single() else {
        warn!("Cannot read space: no local store");
        return;
    };

    // A locally built space is already realized on this entity; reading it back
    // would duplicate every prim and re-run every script. It still has to be
    // served: presence is announced for it, so peers arrive expecting an
    // answer.
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

    commands.entity(trigger.entity).insert(PendingSpacePeers {
        ns,
        sync_targets: sync_targets.0.clone(),
        waiting_since: time.elapsed(),
    });
}

/// Starts the actual space-document fetch once gossip has confirmed an
/// occupant for `ns` (see [`crate::peer::ActiveSpaces`]), or the wait times
/// out. A `NeighborUp` on that space's gossip topic fast-tracks a presence
/// exchange (see `gossip::inbound::handle_gossip_inbound`), so a real
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
    stores: Query<&LocalStore>,
    active_peers: Query<(&Peer, &ActiveSpaces)>,
    mut commands: Commands,
) {
    let Ok(store) = stores.single() else {
        return;
    };

    for (entity, waiting) in &pending {
        let confirmed = active_peers
            .iter()
            .filter(|(_, spaces)| spaces.0.contains_key(&waiting.ns))
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

        spawn_async_task(async move {
            // Waiting on the prim prefix rather than a single snapshot key: the
            // document is its entries now, and the first prim proves it
            // arrived.
            let fetch = async {
                let doc = store.open(ns).await?;
                // Recorded before the content arrives. A space entered and
                // never fully read is still one this node chose to keep.
                store.record_visit(ns).await?;
                doc.start_sync(peers).await?;
                let arrived =
                    wait_for_prefix(&doc, key::PRIM_PREFIX, READ_ATTEMPTS, READ_DELAY).await?;
                anyhow::Ok((doc, arrived))
            };
            tokio::select! {
                () = async { cancel_rx.await.ok(); } => {}
                res = fetch => match res {
                    Ok((doc, true)) => match document::read_state(&doc).await {
                        Ok(state) => {
                            tx.send((doc, state)).await.ok();
                        }
                        Err(err) => error!(?err, "failed reading space entries"),
                    },
                    Ok((_, false)) => warn!(%ns, "space document never arrived"),
                    Err(err) => error!(?err, "failed syncing space document"),
                },
            }
        });

        commands
            .entity(entity)
            .remove::<PendingSpacePeers>()
            .insert(PendingScene {
                rx,
                _cancel: cancel_tx,
            });
    }
}

pub fn instantiate_pending_scenes(
    pending: Query<(Entity, &Space, &PendingScene)>,
    mut commands: Commands,
) {
    for (entity, space, pending) in &pending {
        let Ok((doc, state)) = pending.rx.try_recv() else {
            continue;
        };

        info!(space = %space.0, "Instantiating scene");
        commands
            .entity(entity)
            .insert((
                Hsd::new(state),
                HsdDocId(DocId(*space.0.as_bytes())),
                HsdNamespace(doc),
            ))
            .remove::<PendingScene>();
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
