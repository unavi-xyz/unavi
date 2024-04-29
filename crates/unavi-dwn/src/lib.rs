use bevy::prelude::*;
use dwn::{actor::Actor, store::SurrealStore};
use surrealdb::engine::local::Db;

mod create_record;
pub mod registry;

pub struct DwnPlugin;

impl Plugin for DwnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, create_record::handle_create_record);
    }
}

#[derive(Resource)]
pub struct UserActor(pub Actor<SurrealStore<Db>, SurrealStore<Db>>);
