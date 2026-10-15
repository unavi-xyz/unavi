//! Avian wrapped with the guards a scene or script cannot be trusted
//! without: non-finite or degenerate transforms are parked rather than fed
//! to the solver ([`degenerate`]), a stale collider-tree key is repaired
//! before the physics step reads it ([`collider_tree`]), and a scene is
//! capped at [`PhysicsLimits`] bodies and colliders.

use avian3d::PhysicsPlugins;
use bevy::prelude::*;

mod collider_tree;
pub mod degenerate;
pub mod finite;
pub mod shape;

/// Caps on how much a scene may ask the solver to carry.
///
/// Enforced by `bevy-hsd`, which checks this resource at the one place it
/// inserts `RigidBody`/`Collider` from scene attributes; excess insertions
/// are refused, not queued or truncated.
#[derive(Resource, Clone, Copy, Debug)]
pub struct PhysicsLimits {
    pub max_bodies:    usize,
    pub max_colliders: usize,
}

impl Default for PhysicsLimits {
    fn default() -> Self {
        Self {
            max_bodies:    4096,
            max_colliders: 8192,
        }
    }
}

/// Avian, plus the guards that keep a scene or script from corrupting it.
///
/// Everything that runs physics goes through this rather than adding
/// [`PhysicsPlugins`] directly, so no entry point can be left unguarded.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PhysicsLimits>().add_plugins((
            PhysicsPlugins::default(),
            degenerate::DegenerateBodyPlugin,
            collider_tree::ColliderTreeIntegrityPlugin,
        ));
    }
}
