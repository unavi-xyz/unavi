use bevy::prelude::*;

mod startup;

pub struct SpacePlugin;

impl Plugin for SpacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup::join_home_space);
    }
}
