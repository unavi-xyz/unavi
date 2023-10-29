use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_vrm::VRMPlugin;

pub struct AvatarPlugin;

impl Plugin for AvatarPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(VRMPlugin).add_systems(Startup, spawn_vrm);
    }
}

pub fn spawn_vrm(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Spawning VRM...");

    let mut transform = Transform::from_xyz(-3.0, 0.0, -10.0);
    transform.rotate_y(PI);

    commands.spawn((SceneBundle {
        scene: asset_server.load("catbot.vrm#Scene0"),
        transform,
        ..default()
    },));
}
