use std::{
    sync::Arc,
    time::Duration,
};

use async_channel::Receiver;
use bevy::{
    platform::collections::HashSet,
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
    state::store,
};

/// How long a no-longer-pinned document lingers in the scene before despawn.
const UNPIN_TTL: Duration = Duration::from_mins(3);

const READ_RETRIES: usize = 5;

/// A document instanced into the scene because some peer pins it. Tags only the
/// entities this module spawns, so script-authored docs keep their own
/// lifecycle.
#[derive(Component)]
pub struct PinnedDoc(pub Hash);

#[derive(Component)]
pub struct PendingPinnedDoc {
    rx:      Receiver<LoroDoc>,
    _cancel: oneshot::Sender<()>,
}

#[derive(Component)]
pub struct UnpinnedAt(Duration);

/// Fetches and instances any pinned document not already present in the scene.
pub fn spawn_pinned_docs(
    spaces: Query<(Entity, &Space)>,
    present: Query<&HsdRecordId>,
    pending: Query<&PinnedDoc, With<PendingPinnedDoc>>,
    mut commands: Commands,
) {
    let pinned = store::pinned_docs();
    if pinned.is_empty() {
        return;
    }

    let mut known = present.iter().map(|r| r.0).collect::<HashSet<_>>();
    known.extend(pending.iter().map(|p| p.0));

    for (doc, space) in pinned {
        if known.contains(&doc) {
            continue;
        }
        let Some((space_entity, _)) = spaces.iter().find(|(_, s)| s.0 == space) else {
            continue;
        };

        let (mut event, rx, cancel) = ReadRecord::new(doc);
        event.retries = READ_RETRIES;
        commands.trigger(event);
        commands.spawn((
            PinnedDoc(doc),
            PendingPinnedDoc {
                rx,
                _cancel: cancel,
            },
            ChildOf(space_entity),
        ));
        known.insert(doc);
    }
}

pub fn instantiate_pinned_docs(
    pending: Query<(Entity, &PinnedDoc, &PendingPinnedDoc)>,
    mut commands: Commands,
) {
    for (entity, pin, pending) in &pending {
        let Ok(doc) = pending.rx.try_recv() else {
            continue;
        };
        commands
            .entity(entity)
            .insert((Hsd(Arc::new(doc)), HsdRecordId(pin.0)))
            .remove::<PendingPinnedDoc>();
    }
}

/// Despawns instanced documents once nobody pins them, after [`UNPIN_TTL`].
pub fn despawn_unpinned_docs(
    time: Res<Time>,
    pins: Query<(Entity, &PinnedDoc, Option<&UnpinnedAt>)>,
    mut commands: Commands,
) {
    let pinned = store::pinned_docs()
        .into_iter()
        .map(|(doc, _)| doc)
        .collect::<HashSet<_>>();
    let now = time.elapsed();

    for (entity, pin, unpinned) in &pins {
        if pinned.contains(&pin.0) {
            if unpinned.is_some() {
                commands.entity(entity).remove::<UnpinnedAt>();
            }
            continue;
        }
        match unpinned {
            None => {
                commands.entity(entity).insert(UnpinnedAt(now));
            }
            Some(at) if now.saturating_sub(at.0) >= UNPIN_TTL => {
                commands.entity(entity).despawn();
            }
            Some(_) => {}
        }
    }
}
