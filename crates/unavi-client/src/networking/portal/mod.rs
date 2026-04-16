use bevy::prelude::*;
use blake3::Hash;

#[derive(Component, Clone, Debug)]
#[expect(unused)]
pub struct PortalState {
    pub transform: Transform,
    pub size: Vec2,
    pub dest_space: Hash,
    pub dest_transform: Transform,
    pub dest_size: Vec2,
}

// TODO portal state tracking + instantiation
