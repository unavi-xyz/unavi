use bevy::prelude::*;

mod vrm;

pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<vrm::VRMLoader>()
            .add_systems(Startup, vrm::spawn_vrm);
    }
}
