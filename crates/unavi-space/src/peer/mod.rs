use bevy::{platform::collections::HashMap, prelude::*};
use blake3::Hash;
use iroh::{EndpointAddr, EndpointId};

use crate::state::space::SpaceStateUpdate;

pub mod presence;

#[derive(Component)]
#[require(ActiveSpaces, Transform)]
pub struct Peer(pub EndpointAddr);

#[derive(Component, Default)]
pub struct ActiveSpaces(pub HashMap<Hash, f32>);

#[derive(Event)]
pub struct AddSpaceStateSender {
    pub peer: EndpointId,
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
