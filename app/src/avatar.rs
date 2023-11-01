use bevy::prelude::*;
use bevy_vrm::VRMPlugin;

pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VRMPlugin);
    }
}
