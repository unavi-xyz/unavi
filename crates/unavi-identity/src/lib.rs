use bevy::prelude::*;

mod load;

pub struct IdentityPlugin;

impl Plugin for IdentityPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(load::spawn_actors);
    }
}
