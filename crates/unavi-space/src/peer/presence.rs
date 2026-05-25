use std::{sync::LazyLock, time::Duration};

use async_channel::{Receiver, Sender};
use bevy::{platform::collections::HashMap, prelude::*};
use blake3::Hash;
use iroh::EndpointAddr;

use crate::peer::{ActiveSpaces, Peer};

pub struct PresenceUpdate {
    pub peer: EndpointAddr,
    pub space: Hash,
}

pub const PRESENCE_INTERVAL: Duration = Duration::from_secs(20);
const INACTIVE_SECS: f32 = PRESENCE_INTERVAL.as_secs_f32() * 4.0;

pub static PRESENCE_QUEUE: LazyLock<(Sender<PresenceUpdate>, Receiver<PresenceUpdate>)> =
    LazyLock::new(async_channel::unbounded);

pub fn manage_peers(
    time: Res<Time>,
    mut peers: Query<(Entity, &Peer, &mut ActiveSpaces)>,
    mut commands: Commands,
    mut to_remove: Local<Vec<Hash>>,
) {
    let now = time.elapsed_secs();

    // Refresh active timers.
    while let Ok(update) = PRESENCE_QUEUE.1.try_recv() {
        let Some((entity, _, mut spaces)) =
            peers.iter_mut().find(|(_, p, _)| p.0.id == update.peer.id)
        else {
            let mut spaces = HashMap::default();
            spaces.insert(update.space, now);
            info!("+peer: {}", update.peer.id);
            commands.spawn((Peer(update.peer), ActiveSpaces(spaces)));
            continue;
        };

        spaces.0.insert(update.space, now);

        // Update component with most recent addresses.
        commands.entity(entity).insert(Peer(update.peer));
    }

    // Cull inactive peers.
    if peers.is_empty() {
        return;
    }

    let limit = now - INACTIVE_SECS;

    for (entity, peer, mut spaces) in peers {
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
            info!("-peer: {}", peer.0.id);
            commands.entity(entity).despawn();
        }
    }

    to_remove.shrink_to(4);
}
