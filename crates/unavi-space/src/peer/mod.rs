use std::sync::OnceLock;

use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use blake3::Hash;
use iroh::EndpointAddr;

pub mod presence;
pub mod state;

pub static SELF_PEER: OnceLock<[u8; 32]> = OnceLock::new();

#[must_use]
pub fn self_peer_id() -> Option<[u8; 32]> {
    SELF_PEER.get().copied()
}

#[derive(Component)]
#[require(ActiveSpaces, Transform)]
pub struct Peer(pub EndpointAddr);

#[derive(Component, Default)]
pub struct ActiveSpaces(pub HashMap<Hash, f32>);
