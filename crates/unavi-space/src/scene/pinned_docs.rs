use std::{
    sync::Arc,
    time::Duration,
};

use async_channel::{
    Receiver,
    TryRecvError,
};
use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdRecordId,
};
use bevy_wds::record::read::ReadRecord;
use loro::LoroDoc;
use tokio::sync::oneshot;

use crate::{
    Space,
    peer::Peer,
    state::{
        entities::{
            DocStates,
            SpaceDoc,
        },
        replicas,
    },
};

/// How long an instanced, no-longer-pinned document lingers before despawn.
const UNPIN_TTL: Duration = Duration::from_mins(3);

const READ_RETRIES: usize = 4;

/// The publish path makes a record public and uploads it before announcing the
/// pin, so a holder almost always has it the moment we learn of it. Retries
/// with a short backoff only cover transient connectivity to the holder.
const READ_BACKOFF_SECS: u64 = 1;

/// Delay before re-attempting a fetch whose retries were exhausted.
const REFETCH_DELAY: Duration = Duration::from_secs(10);

#[derive(Component)]
pub struct PendingPinnedDoc {
    rx:      Receiver<LoroDoc>,
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
        if doc.space == space.0 {
            commands.entity(entity).insert(ChildOf(trigger.entity));
        }
    }
}

/// Syncs and instances tracked documents that some peer pins but we lack,
/// once a holder is reachable. Only parented trackers fetch: an unparented one
/// belongs to a space we have not entered, whose content should not instance.
pub fn fetch_tracked_docs(
    time: Res<Time>,
    tracked: Query<
        (Entity, &SpaceDoc, Option<&FetchBackoff>),
        (Without<Hsd>, Without<PendingPinnedDoc>, With<ChildOf>),
    >,
    peers: Query<&Peer>,
    mut commands: Commands,
) {
    let now = time.elapsed();
    for (entity, doc, backoff) in &tracked {
        if backoff.is_some_and(|b| now < b.0) {
            continue;
        }
        if !replicas::is_pinned(doc.doc) {
            continue;
        }

        let holders = replicas::doc_holders(doc.doc);
        let sync_from = holders
            .iter()
            .filter_map(|h| peers.iter().find(|p| p.0.id.as_bytes() == h))
            .map(|p| p.0.clone())
            .collect::<Vec<_>>();
        if sync_from.is_empty() {
            continue;
        }

        let (mut event, rx, cancel) = ReadRecord::new(doc.doc);
        event.retries = READ_RETRIES;
        event.backoff_secs = READ_BACKOFF_SECS;
        event.sync_from = sync_from;
        event.exclusive_sources = true;
        commands.trigger(event);
        commands
            .entity(entity)
            .remove::<FetchBackoff>()
            .insert(PendingPinnedDoc {
                rx,
                _cancel: cancel,
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
            Ok(loro) => {
                commands
                    .entity(entity)
                    .insert((Hsd(Arc::new(loro)), HsdRecordId(doc.doc)))
                    .remove::<PendingPinnedDoc>();
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Closed) => {
                // The tracker anchors replicated peer state (pins, kv), so a
                // failed fetch must not despawn it; retry after a delay.
                warn!("fetch of pinned doc {} failed, retrying", doc.doc);
                commands
                    .entity(entity)
                    .remove::<PendingPinnedDoc>()
                    .insert(FetchBackoff(time.elapsed() + REFETCH_DELAY));
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
    mut commands: Commands,
) {
    let now = time.elapsed();
    for (entity, doc, unpinned) in &instanced {
        if replicas::is_pinned(doc.doc) {
            if unpinned.is_some() {
                commands.entity(entity).remove::<UnpinnedAt>();
            }
        } else if let Some(unpinned) = unpinned {
            if now.saturating_sub(unpinned.0) >= UNPIN_TTL {
                commands.entity(entity).despawn();
            }
        } else {
            commands.entity(entity).insert(UnpinnedAt(now));
        }
    }

    for (entity, states) in &trackers {
        if states.is_none_or(|s| s.iter().next().is_none()) {
            commands.entity(entity).despawn();
        }
    }
}
