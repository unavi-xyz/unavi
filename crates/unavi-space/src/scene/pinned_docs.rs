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
use bevy_iroh::store::LocalStore;
use hsd::state::HsdState;
use iroh::EndpointAddr;
use iroh_docs::NamespaceId;
use tokio::sync::oneshot;
use unavi_policy::space::Space;
use unavi_store::document::Document;

use crate::{
    peer::Peer,
    state::{
        entities::{
            DocStates,
            SpaceDoc,
        },
        replicas::Replicas,
    },
    view::SpaceView,
};

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

/// Reparents unparented document trackers under a space once it is entered, so
/// state replicated for not-yet-visited spaces anchors correctly on join.
pub fn adopt_tracked_docs(
    trigger: On<Add, Space>,
    spaces: Query<&Space>,
    trackers: Query<(Entity, &SpaceDoc), Without<ChildOf>>,
    mut commands: Commands,
) {
    let Ok(space) = spaces.get(trigger.entity) else {
        return;
    };
    for (entity, doc) in &trackers {
        if doc.space == space.doc_id() {
            commands.entity(entity).try_insert(ChildOf(trigger.entity));
        }
    }
}

/// Syncs and instances tracked documents that some peer pins but this client
/// lacks, once a holder is reachable. Only parented trackers fetch: an
/// unparented one belongs to a space that has not been entered, whose content
/// should not instance.
pub fn fetch_tracked_docs(
    time: Res<Time>,
    tracked: Query<
        (Entity, &SpaceDoc, Option<&FetchBackoff>),
        (Without<Hsd>, Without<PendingPinnedDoc>, With<ChildOf>),
    >,
    peers: Query<&Peer>,
    stores: Query<&LocalStore>,
    replicas: Res<Replicas>,
    view: Option<Res<SpaceView>>,
    mut commands: Commands,
) {
    let Ok(store) = stores.single() else {
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
            .holders(doc.doc, me)
            .iter()
            .filter_map(|h| peers.iter().find(|p| &p.0.id == h))
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
                let doc = store.open(ns).await?;
                // Subscribed before the sync starts, so it sees every entry the
                // sync brings.
                let feed = DocFeed::spawn(doc.clone(), FeedReady::RemoteSync);
                doc.start_sync(sync_from).await?;
                anyhow::Ok((doc, feed))
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

pub fn instantiate_tracked_docs(
    time: Res<Time>,
    pending: Query<(Entity, &SpaceDoc, &PendingPinnedDoc)>,
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

/// Reconciles tracked documents to their pin state: instanced docs that go
/// unpinned despawn after [`UNPIN_TTL`]; trackers left with no state at all are
/// dropped immediately.
pub fn prune_tracked_docs(
    time: Res<Time>,
    instanced: Query<(Entity, &SpaceDoc, Option<&UnpinnedAt>), With<Hsd>>,
    trackers: Query<(Entity, Option<&DocStates>), (With<SpaceDoc>, Without<Hsd>)>,
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
