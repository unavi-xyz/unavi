use bevy::prelude::*;
use home::JoinHome;
use wired_protocol::schemas::common::RecordLink;

mod home;
mod scene;
mod skybox;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<JoinHome>()
            .add_systems(Startup, (home::join_home, scene::setup_lights))
            .add_systems(
                Update,
                (
                    home::handle_join_home,
                    scene::create_world_scene,
                    skybox::add_skybox_to_cameras,
                    skybox::process_cubemap,
                ),
            );
    }
}

#[derive(Component)]
pub struct WorldRecord(pub RecordLink);

#[derive(Component)]
pub struct WorldInstance(pub RecordLink);
