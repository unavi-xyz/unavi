use avian3d::PhysicsPlugins;
use bevy::prelude::*;

pub mod body;
mod collider_tree;
pub mod finite;
pub mod shape;

/// Avian, plus the guards that keep a scene or script from corrupting it.
///
/// Everything that runs physics goes through this rather than adding
/// [`PhysicsPlugins`] directly, so no entry point can be left unguarded.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PhysicsPlugins::default(),
            body::DegenerateBodyPlugin,
            collider_tree::ColliderTreeIntegrityPlugin,
        ));
    }
}
