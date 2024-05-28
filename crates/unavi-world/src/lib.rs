use bevy::prelude::*;
use bevy_atmosphere::plugin::{AtmosphereCamera, AtmospherePlugin};
use home::JoinHome;
use wired_social::schemas::common::RecordLink;

mod home;
mod scene;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AtmospherePlugin)
            .add_event::<JoinHome>()
            .add_systems(Startup, (home::join_home, scene::setup_lights))
            .add_systems(
                Update,
                (
                    add_atmosphere_cameras,
                    home::handle_join_home,
                    scene::create_world_scene,
                ),
            );
    }
}

#[derive(Component)]
pub struct WorldRecord(pub RecordLink);

#[derive(Component)]
pub struct InstanceRecord(pub RecordLink);

#[derive(Component)]
pub struct InstanceServer(pub String);

fn add_atmosphere_cameras(
    mut commands: Commands,
    cameras: Query<Entity, (With<Camera3d>, Without<AtmosphereCamera>)>,
) {
    for entity in cameras.iter() {
        commands.entity(entity).insert(AtmosphereCamera::default());
    }
}
