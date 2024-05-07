use bevy::prelude::*;
use dwn::{actor::Actor, store::SurrealStore};
use surrealdb::engine::local::Db;

pub mod create_record;
pub mod query_records;
pub mod world_host;

pub struct DwnPlugin;

impl Plugin for DwnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                create_record::handle_create_record,
                query_records::handle_query_records,
            ),
        );
    }
}

#[derive(Resource)]
pub struct UserActor(pub Actor<SurrealStore<Db>, SurrealStore<Db>>);
