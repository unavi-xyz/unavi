use bevy::prelude::*;
use blake3::Hash;
use smol_str::SmolStr;

pub mod sync;

#[derive(Component, Debug)]
pub struct LocalPlayerState;

#[derive(Component, Debug)]
pub struct RemotePlayerState;

/// Inserted on `LocalPlayerState` when local state changes and a delta should be broadcast.
#[derive(Component, Debug)]
pub struct PlayerStateDirty;

#[derive(Component, Clone, Debug)]
pub struct OwnedObjectEntry {
    pub record_id: Hash,
    pub node_id: SmolStr,
}
