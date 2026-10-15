//! Object sync: the holder of a document streams its dynamic prims, and every
//! other peer drives them as kinematic replicas.

use bevy::prelude::*;
use hsd::id::{
    DocId,
    PrimId,
};

use crate::membership::SpaceId;

pub mod systems;
pub mod wire;

/// One held dynamic prim's space-relative pose, queued for broadcast.
/// Captured once per tick and cloned to each [`ObjectSender`].
#[derive(Clone)]
pub struct OutgoingObject {
    pub doc:   DocId,
    pub space: SpaceId,
    pub prim:  PrimId,
    pub root:  Transform,
    pub lin:   Vec3,
    pub ang:   Vec3,
}

/// Feeds one peer's object streams.
#[derive(Component)]
pub struct ObjectSender(pub async_channel::Sender<Vec<OutgoingObject>>);

/// One prim's update as received, at full precision, checked.
pub struct ResolvedObject {
    pub doc:   DocId,
    pub space: SpaceId,
    pub prim:  PrimId,
    pub root:  Transform,
    pub lin:   Vec3,
    pub ang:   Vec3,
}
