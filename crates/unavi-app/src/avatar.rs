use bevy::prelude::*;
use bevy_vrm::VrmPlugin;

pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VrmPlugin);
    }
}
