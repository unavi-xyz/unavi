use bevy::prelude::*;

pub mod async_commands;

pub struct UtilPlugin;

impl Plugin for UtilPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, async_commands::apply_async_commands);
    }
}
