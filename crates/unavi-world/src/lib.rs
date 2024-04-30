use bevy::prelude::*;
use home::JoinHome;
use wired_protocol::schemas::common::RecordLink;

mod home;
mod scene;
mod skybox;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<WorldState>()
            .add_event::<JoinHome>()
            .add_systems(Startup, home::join_home)
            .add_systems(
                OnEnter(WorldState::InWorld),
                (scene::setup_scene, skybox::create_skybox),
            )
            .add_systems(
                Update,
                (
                    home::handle_join_home,
                    (skybox::add_skybox_to_cameras, skybox::process_cubemap)
                        .run_if(in_state(WorldState::InWorld)),
                ),
            );
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum WorldState {
    #[default]
    LoadingWorld,
    InWorld,
}

#[derive(Component)]
pub struct WorldRecord(pub RecordLink);

#[derive(Component)]
pub struct WorldInstance(pub RecordLink);
