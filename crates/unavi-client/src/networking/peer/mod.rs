use std::collections::HashSet;

use bevy::prelude::*;
use blake3::Hash;
use iroh::EndpointId;

pub mod state;

#[derive(Component, Debug)]
pub struct Peer(pub EndpointId);

#[derive(Component, Debug, Default)]
pub struct PeerKnownSpaces(pub HashSet<Hash>);

#[derive(Component, Debug, Default, PartialEq, Eq)]
pub enum PeerStateStatus {
    #[default]
    NeverSynced,
    Requested,
    Synced,
    #[expect(unused)]
    NeedsResync,
}
