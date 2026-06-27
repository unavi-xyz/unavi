use std::{
    sync::Arc,
    time::Duration,
};

use async_channel::{
    Receiver,
    TryRecvError,
};
use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_hsd::{
    Hsd,
    HsdRecordId,
};
use bevy_wds::record::read::ReadRecord;
use blake3::Hash;
use loro::LoroDoc;
use tokio::sync::oneshot;

use crate::{
    Space,
    peer::Peer,
    state::peer::{
        self,
        PinChange,
    },
};

/// How long a no-longer-pinned document lingers in the scene before despawn.
const UNPIN_TTL: Duration = Duration::from_mins(3);

const READ_RETRIES: usize = 4;

/// The publish path makes a record public and uploads it before announcing the
/// pin, so a holder almost always has it the moment we learn of it. Retries
/// with a short backoff only cover transient connectivity to the holder.
const READ_BACKOFF_SECS: u64 = 1;

/// A document instanced into the scene because some peer pins it. Tags only the
/// entities this module spawns, so script-authored docs keep their own
/// lifecycle.
#[derive(Component)]
pub struct PinnedDoc(pub Hash);

/// A pinned document we lack and must sync from a holder once one is reachable.
#[derive(Component)]
pub struct FetchPinnedDoc {
    space: Hash,
}

#[derive(Component)]
pub struct PendingPinnedDoc {
    rx:      Receiver<LoroDoc>,
    _cancel: oneshot::Sender<()>,
}

#[derive(Component)]
pub struct UnpinnedAt(Duration);

/// Receives pin lifecycle transitions from the peer state store.
#[derive(Resource)]
pub struct PinChanges(pub Receiver<PinChange>);

/// Reconciles the scene to pin transitions: a doc's first holder spawns a
/// fetch, its last holder schedules despawn after [`UNPIN_TTL`].
pub fn apply_pin_changes(
    changes: Res<PinChanges>,
    pins: Query<(Entity, &PinnedDoc)>,
    present: Query<&HsdRecordId>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let mut events = Vec::new();
    while let Ok(change) = changes.0.try_recv() {
        events.push(change);
    }
    if events.is_empty() {
        return;
    }

    let existing = pins
        .iter()
        .map(|(e, p)| (p.0, e))
        .collect::<HashMap<_, _>>();
    let now = time.elapsed();
    for change in events {
        match change {
            PinChange::Pinned { doc, space } => {
                if let Some(entity) = existing.get(&doc) {
                    commands.entity(*entity).remove::<UnpinnedAt>();
                } else if !present.iter().any(|r| r.0 == doc) {
                    commands.spawn((PinnedDoc(doc), FetchPinnedDoc { space }));
                }
            }
            PinChange::Unpinned { doc } => {
                if let Some(entity) = existing.get(&doc) {
                    commands.entity(*entity).insert(UnpinnedAt(now));
                }
            }
        }
    }
}

/// Syncs and instances documents awaiting a reachable holder, parenting them
/// under their space.
pub fn fetch_pinned_docs(
    to_fetch: Query<(Entity, &PinnedDoc, &FetchPinnedDoc)>,
    spaces: Query<(Entity, &Space)>,
    peers: Query<&Peer>,
    mut commands: Commands,
) {
    for (entity, pin, fetch) in &to_fetch {
        let Some((space_entity, _)) = spaces.iter().find(|(_, s)| s.0 == fetch.space) else {
            continue;
        };

        let holders = peer::doc_holders(pin.0);
        let sync_from = holders
            .iter()
            .filter_map(|h| peers.iter().find(|p| p.0.id.as_bytes() == h))
            .map(|p| p.0.clone())
            .collect::<Vec<_>>();
        if sync_from.is_empty() {
            continue;
        }

        let (mut event, rx, cancel) = ReadRecord::new(pin.0);
        event.retries = READ_RETRIES;
        event.backoff_secs = READ_BACKOFF_SECS;
        event.sync_from = sync_from;
        event.exclusive_sources = true;
        commands.trigger(event);
        commands
            .entity(entity)
            .insert((
                PendingPinnedDoc {
                    rx,
                    _cancel: cancel,
                },
                ChildOf(space_entity),
            ))
            .remove::<FetchPinnedDoc>();
    }
}

pub fn instantiate_pinned_docs(
    pending: Query<(Entity, &PinnedDoc, &PendingPinnedDoc)>,
    mut commands: Commands,
) {
    for (entity, pin, pending) in &pending {
        match pending.rx.try_recv() {
            Ok(doc) => {
                commands
                    .entity(entity)
                    .insert((Hsd(Arc::new(doc)), HsdRecordId(pin.0)))
                    .remove::<PendingPinnedDoc>();
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Closed) => {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Despawns instanced documents once they have gone unpinned for [`UNPIN_TTL`].
pub fn despawn_unpinned_docs(
    time: Res<Time>,
    pins: Query<(Entity, &UnpinnedAt)>,
    mut commands: Commands,
) {
    let now = time.elapsed();
    for (entity, unpinned) in &pins {
        if now.saturating_sub(unpinned.0) >= UNPIN_TTL {
            commands.entity(entity).despawn();
        }
    }
}
