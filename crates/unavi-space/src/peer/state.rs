use bevy::prelude::*;
use iroh::EndpointId;

use crate::{
    peer::{
        ActiveSpaces,
        Peer,
    },
    state::space::SpaceStateUpdate,
};

#[derive(Event)]
pub struct AddSpaceStateSender {
    pub peer:   EndpointId,
    pub sender: async_channel::Sender<SpaceStateUpdate>,
}

#[derive(Component)]
pub struct SpaceStateSender(pub async_channel::Sender<SpaceStateUpdate>);

pub fn add_space_state_sender(
    trigger: On<AddSpaceStateSender>,
    peers: Query<(Entity, &Peer)>,
    mut commands: Commands,
) {
    let Some((entity, _)) = peers.iter().find(|(_, p)| p.0.id == trigger.peer) else {
        return;
    };

    commands
        .entity(entity)
        .insert(SpaceStateSender(trigger.sender.clone()));
}

pub fn publish_state_update(
    trigger: On<SpaceStateUpdate>,
    peers: Query<(&ActiveSpaces, &SpaceStateSender)>,
) {
    for (spaces, sender) in peers {
        if !spaces.0.contains_key(&trigger.space) {
            continue;
        }
        let _ = sender.0.try_send(trigger.clone());
    }
}
