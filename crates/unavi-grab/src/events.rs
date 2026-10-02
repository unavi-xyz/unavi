//! What a grab does to an entity, told to whoever wants to react to it
//! without knowing how a grab works.

use bevy::prelude::*;

/// A pointer took hold of `entity`.
///
/// Fired once the grab has already taken effect physically (gravity
/// zeroed, velocity now pointer-driven); an observer deciding whether the
/// grabber may claim anything beyond physics — a document's hold, say —
/// runs after the fact and undoes nothing here if it refuses.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct Grabbed {
    pub entity:  Entity,
    pub pointer: Entity,
}

/// `entity` was let go: by its own release, or because its pointer vanished.
/// Physics has already been undone (gravity restored) by the time this
/// fires.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct Released {
    pub entity:  Entity,
    pub pointer: Entity,
}
