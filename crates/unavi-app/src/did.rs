use bevy::prelude::*;
use bevy_async_task::AsyncTaskRunner;
use dwn::{
    actor::{Actor, MessageBuilder},
    message::descriptor::records::Version,
    store::SurrealStore,
};
use serde_json::Value;
use surrealdb::engine::local::Db;

const REGISTRY_DID: &str = "did:web:localhost%3A3000";
const REGISTRY_VERSION: &str = "0.0.1";
const REGISTRY_PATH: &str = "world";
const PROTOCOL_DEFINITION: &str =
    include_str!("../../../wired-protocol/social/dwn/protocols/world-registry.json");

pub struct DidPlugin;

impl Plugin for DidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_world);
    }
}

#[derive(Resource)]
pub struct UserActor(pub Actor<SurrealStore<Db>, SurrealStore<Db>>);

fn create_world(actor: Res<UserActor>, mut task_runner: AsyncTaskRunner<()>) {
    let value: Value = serde_json::from_str(PROTOCOL_DEFINITION).unwrap();
    let protocol = value["protocol"].as_str().unwrap().to_string();

    let actor = actor.0.clone();

    task_runner.start(async move {
        let mut create = actor.create_record().published(true).protocol(
            protocol,
            Version::parse(REGISTRY_VERSION).unwrap(),
            REGISTRY_PATH.to_string(),
        );

        let reply = create.send(REGISTRY_DID).await;

        info!("Created world: {:#?}", reply);
    });
}
