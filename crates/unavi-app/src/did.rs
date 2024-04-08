use bevy::prelude::*;
use bevy_async_task::AsyncTaskRunner;
use dwn::{
    actor::{Actor, MessageBuilder},
    store::SurrealStore,
};
use surrealdb::engine::local::Db;
use wired_protocol::registry::{registry_protocol, REGISTRY_PROTOCOL_VERSION};

const REGISTRY_DID: &str = "did:web:localhost%3A3000";

pub struct DidPlugin;

impl Plugin for DidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_world);
    }
}

#[derive(Resource)]
pub struct UserActor(pub Actor<SurrealStore<Db>, SurrealStore<Db>>);

fn create_world(actor: Res<UserActor>, mut task_runner: AsyncTaskRunner<()>) {
    let actor = actor.0.clone();

    task_runner.start(async move {
        let mut create = actor.create_record().published(true).protocol(
            registry_protocol(),
            REGISTRY_PROTOCOL_VERSION,
            "world".to_string(),
        );

        let reply = create.send(REGISTRY_DID).await;

        info!("Created world: {:#?}", reply);
    });
}
