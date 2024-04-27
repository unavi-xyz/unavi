use bevy::prelude::*;
use bevy_async_task::{AsyncTaskRunner, AsyncTaskStatus};
use dwn::{
    actor::{Actor, MessageBuilder, ProcessMessageError},
    message::descriptor::iana_media_types::{Application, MediaType},
    store::SurrealStore,
    MessageReply,
};
use surrealdb::engine::local::Db;
use wired_protocol::{
    protocols::world_registry::{
        registry_definition, registry_world_schema_url, REGISTRY_PROTOCOL_VERSION,
    },
    schemas::world::World,
};

const ENV_REGISTRY_DID: Option<&str> = option_env!("UNAVI_REGISTRY_DID");
const LOCAL_REGISTRY_DID: &str = "did:web:localhost%3A3000";

fn registry_did() -> &'static str {
    ENV_REGISTRY_DID.unwrap_or(LOCAL_REGISTRY_DID)
}

pub struct DidPlugin;

impl Plugin for DidPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreateWorld>()
            .add_systems(Startup, create_initial_world)
            .add_systems(Update, create_worlds);
    }
}

#[derive(Resource)]
pub struct UserActor(pub Actor<SurrealStore<Db>, SurrealStore<Db>>);

#[derive(Event)]
pub struct CreateWorld(pub World);

fn create_initial_world(mut _writer: EventWriter<CreateWorld>) {
    // writer.send(CreateWorld(World {
    //     name: Some("New World".to_string()),
    //     ..Default::default()
    // }));
}

fn create_worlds(
    actor: Res<UserActor>,
    mut events: EventReader<CreateWorld>,
    mut task: AsyncTaskRunner<Result<MessageReply, ProcessMessageError>>,
) {
    match task.poll() {
        AsyncTaskStatus::Idle => {
            for event in events.read() {
                let actor = actor.0.clone();
                let world = event.0.clone();

                task.start(async move {
                    let did = registry_did();
                    info!("Creating world at registry: {}", did);

                    actor
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
                        .target(did.to_string())
                        .send(did)
                        .await
                });
            }
        }
        AsyncTaskStatus::Pending => {}
        AsyncTaskStatus::Finished(res) => {
            match res {
                Ok(reply) => info!("Created world: {:#?}", reply),
                Err(err) => error!("Failed to create world: {}", err),
            };
        }
    };
}
