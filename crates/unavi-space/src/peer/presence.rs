use std::time::Duration;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use hsd::id::DocId;
use iroh::{
    EndpointAddr,
    EndpointId,
};
use iroh_docs::NamespaceId;

use crate::{
    inbox::Inbox,
    peer::{
        ActiveSpaces,
        Peer,
    },
};

pub const PRESENCE_INTERVAL: Duration = Duration::from_secs(20);
const INACTIVE_SECS: f32 = PRESENCE_INTERVAL.as_secs_f32() * 4.0;

/// Presence broadcasts heard over gossip, handed from each space's inbound
/// task to the ECS.
#[derive(Resource, Clone, Default)]
pub struct PresenceInbox(Inbox<(EndpointId, DocId), EndpointAddr>);

impl PresenceInbox {
    #[must_use]
    pub fn inbox(&self) -> Inbox<(EndpointId, DocId), EndpointAddr> {
        self.0.clone()
    }
}

pub fn manage_peers(
    time: Res<Time>,
    presence: Res<PresenceInbox>,
    mut peers: Query<(Entity, &Peer, &mut ActiveSpaces)>,
    mut commands: Commands,
    mut to_remove: Local<Vec<NamespaceId>>,
) {
    let now = time.elapsed_secs();

    let updates = presence.0.drain();

    for ((_, space), peer) in updates {
        let space = NamespaceId::from(&space.0);
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
