use bevy::prelude::*;
use dwn::{actor::Actor, store::SurrealStore};
use home::JoinHome;
use surrealdb::engine::local::Db;
use wired_social::schemas::common::RecordLink;

mod home;
mod loading;
mod scene;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(target_family = "wasm"))]
        app.add_plugins(bevy_atmosphere::plugin::AtmospherePlugin)
            .add_systems(Update, add_atmosphere_cameras);

        app.add_event::<JoinHome>()
            .init_state::<WorldState>()
            .add_systems(Startup, (home::join_home, scene::setup_lights))
            .add_systems(
                Update,
                (
                    home::handle_join_home,
                    scene::create_world_scene,
                    loading::set_loading_state,
                ),
            )
            .add_systems(OnEnter(WorldState::Loading), loading::spawn_loading_scene)
            .add_systems(OnExit(WorldState::Loading), loading::despawn_loading_scene);
    }
}

#[derive(States, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum WorldState {
    InWorld,
    #[default]
    Loading,
}

#[derive(Component)]
pub struct WorldRecord(pub RecordLink);

#[derive(Component)]
pub struct InstanceRecord(pub RecordLink);

#[derive(Component)]
pub struct InstanceServer(pub String);

#[cfg(not(target_family = "wasm"))]
fn add_atmosphere_cameras(
    mut commands: Commands,
    cameras: Query<
        Entity,
        (
            With<Camera3d>,
            Without<bevy_atmosphere::plugin::AtmosphereCamera>,
        ),
    >,
) {
    for entity in cameras.iter() {
        commands
            .entity(entity)
            .insert(bevy_atmosphere::plugin::AtmosphereCamera::default());
    }
}

#[derive(Resource)]
pub struct UserActor(pub Actor<SurrealStore<Db>, SurrealStore<Db>>);
