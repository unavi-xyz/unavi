use bevy::{
    platform::collections::HashMap,
    prelude::*,
};
use blake3::Hash;
use iroh::EndpointAddr;

pub mod presence;
pub mod state;

#[derive(Component)]
#[require(ActiveSpaces, Transform)]
pub struct Peer(pub EndpointAddr);

#[derive(Component, Default)]
pub struct ActiveSpaces(pub HashMap<Hash, f32>);
