use std::{
    mem,
    sync::{LazyLock, Mutex},
    time::Duration,
};

use bevy::{platform::collections::HashMap, prelude::*};
use blake3::Hash;
use iroh::{EndpointAddr, EndpointId};

use crate::peer::{ActiveSpaces, Peer};

pub const PRESENCE_INTERVAL: Duration = Duration::from_secs(20);
const INACTIVE_SECS: f32 = PRESENCE_INTERVAL.as_secs_f32() * 4.0;

type PresenceKey = (EndpointId, Hash);

static PRESENCE_INBOX: LazyLock<Mutex<HashMap<PresenceKey, EndpointAddr>>> =
    LazyLock::new(|| Mutex::new(HashMap::default()));

pub fn submit_presence(peer: EndpointAddr, space: Hash) {
    let mut inbox = PRESENCE_INBOX.lock().expect("presence inbox");
    inbox.insert((peer.id, space), peer);
}

pub fn manage_peers(
    time: Res<Time>,
    mut peers: Query<(Entity, &Peer, &mut ActiveSpaces)>,
    mut commands: Commands,
    mut to_remove: Local<Vec<Hash>>,
) {
    let now = time.elapsed_secs();

    let updates = mem::take(&mut *PRESENCE_INBOX.lock().expect("presence inbox"));

    for ((_, space), peer) in updates {
        let Some((entity, _, mut spaces)) = peers.iter_mut().find(|(_, p, _)| p.0.id == peer.id)
        else {
            let mut spaces = HashMap::default();
            spaces.insert(space, now);
            info!("+peer: {}", peer.id);
            commands.spawn((Peer(peer), ActiveSpaces(spaces)));
            continue;
        };

        spaces.0.insert(space, now);
        commands.entity(entity).insert(Peer(peer));
    }

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
