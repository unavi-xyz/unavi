use std::{
    sync::{LazyLock, Mutex},
    time::Duration,
};

use bevy::{platform::collections::HashMap, prelude::*};
use blake3::Hash;
use iroh::EndpointId;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{ActiveSpaces, Peer};

pub struct PresenceUpdate {
    pub peer: EndpointId,
    pub space: Hash,
}

pub const PRESENCE_INTERVAL: Duration = Duration::from_secs(20);
const INACTIVE_SECS: f32 = PRESENCE_INTERVAL.as_secs_f32() * 4.0;

const SIZE: usize = 32;

pub static PRESENCE_QUEUE: LazyLock<(Sender<PresenceUpdate>, Mutex<Receiver<PresenceUpdate>>)> =
    LazyLock::new(|| {
        let (tx, rx) = tokio::sync::mpsc::channel(SIZE);
        (tx, Mutex::new(rx))
    });

pub fn manage_peers(
    time: Res<Time>,
    mut peers: Query<(Entity, &Peer, &mut ActiveSpaces)>,
    mut commands: Commands,
    mut to_remove: Local<Vec<Hash>>,
) {
    let Ok(mut guard) = PRESENCE_QUEUE.1.try_lock() else {
        return;
    };

    let now = time.elapsed_secs();

    // Refresh active timers.
    while let Ok(update) = guard.try_recv() {
        let Some((_, _, mut spaces)) = peers.iter_mut().find(|(_, p, _)| p.0 == update.peer) else {
            let mut spaces = HashMap::default();
            spaces.insert(update.space, now);
            info!("new peer: {}", update.peer);
            commands.spawn((Peer(update.peer), ActiveSpaces(spaces)));
            continue;
        };

        spaces.0.insert(update.space, now);
    }

    // Cull inactive peers.
    if peers.is_empty() {
        return;
    }

    let limit = now - INACTIVE_SECS;

    for (entity, _, mut spaces) in peers {
        for (space, t) in &spaces.0 {
            if *t > limit {
                continue;
            }

            to_remove.push(*space);
        }

        for space in to_remove.drain(..) {
            spaces.0.remove(&space);
        }

        if spaces.0.is_empty() {
            commands.entity(entity).despawn();
        }
    }

    to_remove.shrink_to(4);
}
