use bevy::prelude::*;

mod home;
mod skybox;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<WorldState>()
            .add_systems(Startup, home::load_home)
            .add_systems(
                OnEnter(WorldState::InWorld),
                (home::setup_world, skybox::create_skybox),
            )
            .add_systems(
                Update,
                (skybox::add_skybox_to_cameras, skybox::process_cubemap)
                    .run_if(in_state(WorldState::InWorld)),
            );
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, States)]
pub enum WorldState {
    #[default]
    LoadingWorld,
    InWorld,
}
