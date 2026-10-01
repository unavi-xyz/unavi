//! Documents peers pin into a space: fetched once a pinner is reachable,
//! instanced under the space, and despawned a while after the last pin goes.

use std::time::Duration;

use async_channel::{
    Receiver,
    TryRecvError,
};
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
use hsd::{
    id::DocId,
    state::HsdState,
};
use iroh::EndpointAddr;
use iroh_docs::NamespaceId;
use tokio::sync::oneshot;
use unavi_store::Document;

use crate::{
    authority::SpaceView,
    discovery::Peer,
    index::{
        self,
        Index,
        Indexed,
    },
    membership::{
        Space,
        SpaceId,
    },
    replication::{
        Replicas,
        guards::DocGuards,
    },
};

/// A document tracked because some peer references it, anchoring its guards.
/// Parented under the [`Space`] so leaving the space cascades it away;
/// unparented until the space is entered and adopts it.
#[derive(Component)]
#[component(
    immutable,
    on_insert = index::insert::<Self>,
    on_discard = index::discard::<Self>
)]
pub struct PinnedDoc {
    pub doc:   DocId,
    pub space: SpaceId,
}

impl Indexed for PinnedDoc {
    type Key = DocId;

    fn key(&self) -> DocId {
        self.doc
    }
}

/// How long an instanced, no-longer-pinned document lingers before despawn.
const UNPIN_TTL: Duration = Duration::from_mins(3);

/// Delay before re-attempting a fetch whose retries were exhausted.
const REFETCH_DELAY: Duration = Duration::from_secs(10);

#[derive(Component)]
pub struct PendingPinnedDoc {
    /// Carries the document alongside its feed, so the component that lands
    /// on the entity holds the same handle the fetch opened.
    rx:      Receiver<(Document, DocFeed)>,
    peers:   Vec<EndpointAddr>,
    _cancel: oneshot::Sender<()>,
}

/// Earliest time the next fetch attempt may run, set after a failed fetch.
#[derive(Component)]
pub struct FetchBackoff(Duration);

#[derive(Component)]
pub struct UnpinnedAt(Duration);

/// Reparents unparented trackers under a space once it is entered, so state
/// replicated for not-yet-visited spaces anchors correctly on join.
pub fn adopt_pinned_docs(
    trigger: On<Add, Space>,
    spaces: Query<&Space>,
    trackers: Query<(Entity, &PinnedDoc), Without<ChildOf>>,
    mut commands: Commands,
) {
    let Ok(space) = spaces.get(trigger.entity) else {
        return;
    };
    for (entity, doc) in &trackers {
        if doc.space == space.id() {
            commands.entity(entity).try_insert(ChildOf(trigger.entity));
        }
    }
}

/// Syncs and instances pinned documents this client lacks, once a pinner is
/// reachable.
///
/// Only parented trackers fetch: an unparented one belongs to a space that has
/// not been entered, whose content should not instance.
pub fn fetch_pinned_docs(
    time: Res<Time>,
    tracked: Query<
        (Entity, &PinnedDoc, Option<&FetchBackoff>),
        (Without<Hsd>, Without<PendingPinnedDoc>, With<ChildOf>),
    >,
    peers: Query<&Peer>,
    peer_index: Res<Index<Peer>>,
    store: Option<Res<DataStore>>,
    replicas: Res<Replicas>,
    view: Option<Res<SpaceView>>,
    mut commands: Commands,
) {
    let Some(store) = store else {
        return;
    };
    // A doc with no local identity yet has nothing to sync into; retried next
    // tick once one exists.
    let Some(me) = view.as_deref().map(SpaceView::me) else {
        return;
    };
    let now = time.elapsed();
    for (entity, doc, backoff) in &tracked {
        if backoff.is_some_and(|b| now < b.0) {
            continue;
        }
        if !replicas.is_pinned(doc.doc) {
            continue;
        }

        let sync_from = replicas
            .sync_sources(doc.doc, me)
            .into_iter()
            .filter_map(|id| peers.get(peer_index.get(id)?).ok())
            .map(|p| p.0.clone())
            .collect::<Vec<_>>();
        if sync_from.is_empty() {
            continue;
        }

        let ns = NamespaceId::from(&doc.doc.0);
        let store = store.0.clone();
        let (tx, rx) = async_channel::bounded(1);
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let peers = sync_from.clone();

        task::spawn(async move {
            let fetch = async {
                let (doc, events) = store.join(ns, sync_from).await?;
                let feed = DocFeed::joined(doc.clone(), events, FeedReady::RemoteSync);
                Ok::<_, unavi_store::Error>((doc, feed))
            };
            tokio::select! {
                () = async { cancel_rx.await.ok(); } => {}
                res = fetch => match res {
                    Ok(fetched) => {
                        if tx.send(fetched).await.is_err() {
                            debug!(%ns, "pinned document fetched after its tracker was dropped");
                        }
                    }
                    Err(err) => warn!(%ns, ?err, "failed syncing pinned document"),
                },
            }
        });

        commands
            .entity(entity)
            .try_remove::<FetchBackoff>()
            .try_insert(PendingPinnedDoc {
                rx,
                peers,
                _cancel: cancel_tx,
            });
    }
}

pub fn instance_pinned_docs(
    time: Res<Time>,
    pending: Query<(Entity, &PinnedDoc, &PendingPinnedDoc)>,
    mut commands: Commands,
) {
    for (entity, doc, pending) in &pending {
        match pending.rx.try_recv() {
            Ok((namespace, feed)) => {
                commands
                    .entity(entity)
                    .try_insert((
                        Hsd::new(HsdState::new()),
                        HsdDocId(doc.doc),
                        HsdNamespace(namespace),
                        SyncPeers(pending.peers.clone()),
                        feed,
                    ))
                    .try_remove::<PendingPinnedDoc>();
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Closed) => {
                // The tracker anchors replicated peer state (pins, kv), so a
                // failed fetch must not despawn it; retry after a delay.
                warn!("fetch of pinned doc {} failed, retrying", doc.doc);
                commands
                    .entity(entity)
                    .try_remove::<PendingPinnedDoc>()
                    .try_insert(FetchBackoff(time.elapsed() + REFETCH_DELAY));
            }
        }
    }
}

/// Reconciles trackers to their pin state: instanced docs that go unpinned
/// despawn after [`UNPIN_TTL`]; trackers left with no state at all are dropped
/// immediately.
pub fn prune_pinned_docs(
    time: Res<Time>,
    instanced: Query<(Entity, &PinnedDoc, Option<&UnpinnedAt>), With<Hsd>>,
    trackers: Query<(Entity, Option<&DocGuards>), (With<PinnedDoc>, Without<Hsd>)>,
    replicas: Res<Replicas>,
    mut commands: Commands,
) {
    let now = time.elapsed();
    for (entity, doc, unpinned) in &instanced {
        if replicas.is_pinned(doc.doc) {
            if unpinned.is_some() {
                commands.entity(entity).remove::<UnpinnedAt>();
            }
        } else if let Some(unpinned) = unpinned {
            if now.saturating_sub(unpinned.0) >= UNPIN_TTL {
                commands.entity(entity).despawn();
            }
        } else {
            commands.entity(entity).try_insert(UnpinnedAt(now));
        }
    }

    for (entity, states) in &trackers {
        if states.is_none_or(|s| s.iter().next().is_none()) {
            commands.entity(entity).despawn();
        }
    }
}
