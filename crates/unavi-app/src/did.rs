use bevy::prelude::*;
use bevy_async_task::AsyncTask;
use dwn::{
    actor::{Actor, MessageBuilder},
    message::descriptor::iana_media_types::{Application, MediaType},
    store::SurrealStore,
};
use surrealdb::engine::local::Db;
use wired_protocol::registry::{
    registry_definition, registry_world_schema_url, World, REGISTRY_PROTOCOL_VERSION,
};

const REGISTRY_DID: &str = "did:web:localhost%3A3000";

pub struct DidPlugin;

impl Plugin for DidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_world);
    }
}

#[derive(Resource)]
pub struct UserActor(pub Actor<SurrealStore<Db>, SurrealStore<Db>>);

fn create_world(actor: Res<UserActor>) {
    let actor = actor.0.clone();

    AsyncTask::new(async move {
        let world = World {
            name: Some("New World".to_string()),
            ..Default::default()
        };

        match actor
            .create_record()
            .data(serde_json::to_vec(&world).unwrap())
            .data_format(MediaType::Application(Application::Json))
            .schema(registry_world_schema_url())
            .protocol(
                registry_definition().protocol,
                REGISTRY_PROTOCOL_VERSION,
                "world".to_string(),
            )
            .published(true)
            .target(REGISTRY_DID.to_string())
            .send(REGISTRY_DID)
            .await
        {
            Ok(reply) => info!("Created world: {:#?}", reply),
            Err(err) => {
                error!("Failed to create world: {:#?}", err)
            }
        };
    })
    .blocking_recv();
}
