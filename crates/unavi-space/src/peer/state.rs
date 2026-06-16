use bevy::prelude::*;

use crate::{
    connection::ecs::PeerStream,
    peer::{
        ActiveSpaces,
        Peer,
    },
    state::space::SpaceStateUpdate,
};

#[derive(Component)]
pub struct SpaceStateSender(pub async_channel::Sender<SpaceStateUpdate>);

pub fn publish_state_update(
    trigger: On<SpaceStateUpdate>,
    streams: Query<(&PeerStream, &SpaceStateSender)>,
    peers: Query<(&Peer, &ActiveSpaces)>,
) {
    for (stream, sender) in streams {
        let Some((_, spaces)) = peers.iter().find(|(p, _)| p.0.id == stream.0) else {
            continue;
        };
        if !spaces.0.contains_key(&trigger.space) {
            continue;
        }
        if let Err(err) = sender.0.try_send(trigger.clone()) {
            error!(?err, "State send error");
        }
    }
}
