use bevy::prelude::*;
use dwn::{actor::Actor, store::SurrealStore};
use surrealdb::engine::local::Db;

mod home;
mod instance;
mod registry;
mod world;

pub struct DwnPlugin;

impl Plugin for DwnPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<instance::CreateInstance>().add_systems(
            Update,
            (world::handle_create_world, instance::handle_create_instance),
        );
    }
}

#[derive(Resource)]
pub struct UserActor(pub Actor<SurrealStore<Db>, SurrealStore<Db>>);
