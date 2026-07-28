use std::sync::RwLock;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use bevy_wds::LocalActor;
use blake3::Hash;
use iroh::EndpointAddr;

pub mod presence;

pub static SELF_PEER: RwLock<Option<[u8; 32]>> = RwLock::new(None);
pub static SELF_DID: RwLock<Option<String>> = RwLock::new(None);

#[must_use]
pub fn self_peer_id() -> Option<[u8; 32]> {
    *SELF_PEER.read().expect("SELF_PEER poisoned")
}

pub fn set_self_peer_id(peer: [u8; 32]) {
    let mut current = SELF_PEER.write().expect("SELF_PEER poisoned");
    if let Some(existing) = *current
        && existing != peer
    {
        info!("self peer id changed (endpoint re-created)");
    }
    *current = Some(peer);
}

#[must_use]
pub fn self_did() -> Option<String> {
    SELF_DID.read().expect("SELF_DID poisoned").clone()
}

pub fn capture_self_did(trigger: On<Add, LocalActor>, actors: Query<&LocalActor>) {
    if let Ok(actor) = actors.get(trigger.entity) {
        *SELF_DID.write().expect("SELF_DID poisoned") =
            Some(actor.0.identity().did().to_string());
    }
}

#[derive(Component)]
#[require(ActiveSpaces, Transform)]
pub struct Peer(pub EndpointAddr);

#[derive(Component, Default)]
pub struct ActiveSpaces(pub HashMap<Hash, f32>);
