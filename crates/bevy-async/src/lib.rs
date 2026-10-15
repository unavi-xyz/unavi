//! Bridges async tasks and a Bevy `App`.
//!
//! [`task::spawn`] runs a future on a shared runtime. A task reaches the world
//! through [`AsyncCommands`], minted from the [`AsyncWorld`] resource of the
//! `App` it should act on. The queue lives in that resource, so two `App`s in
//! one process never see each other's commands.

use bevy::prelude::*;

pub mod commands;
pub mod task;

pub use commands::{
    AsyncCommands,
    AsyncWorld,
};

/// Orders systems against the point where queued async commands apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct AsyncSystems;

/// Inserts [`AsyncWorld`] and applies what tasks queue at the start of every
/// frame, ahead of `Update`.
pub struct AsyncPlugin;

impl Plugin for AsyncPlugin {
    fn build(&self, app: &mut App) {
        let (async_world, inbox) = commands::queue();
        app.insert_resource(async_world)
            .insert_resource(inbox)
            .add_systems(PreUpdate, commands::apply.in_set(AsyncSystems));
    }
}
