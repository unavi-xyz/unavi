use bevy::prelude::*;
use bevy_async_task::{AsyncTaskRunner, AsyncTaskStatus};
use dwn::{
    actor::{MessageBuilder, ProcessMessageError},
    message::descriptor::iana_media_types::{Application, MediaType},
    MessageReply,
};
use wired_protocol::{
    protocols::world_registry::{registry_definition, REGISTRY_PROTOCOL_VERSION},
    schemas::instance::{instance_schema_url, Instance},
};

use crate::{registry::registry_did, UserActor};

#[derive(Event)]
pub struct CreateInstance(pub Instance);

pub fn handle_create_instance(
    actor: Res<UserActor>,
    mut events: EventReader<CreateInstance>,
    mut task: AsyncTaskRunner<Result<MessageReply, ProcessMessageError>>,
) {
    match task.poll() {
        AsyncTaskStatus::Idle => {
            for event in events.read() {
                let actor = actor.0.clone();
                let instance = event.0.clone();

                task.start(async move {
                    let did = registry_did();
                    info!("Creating instance at registry: {}", did);

                    actor
                        .create_record()
                        .data(serde_json::to_vec(&instance).unwrap())
                        .data_format(MediaType::Application(Application::Json))
                        .schema(instance_schema_url())
                        .protocol(
                            registry_definition().protocol,
                            REGISTRY_PROTOCOL_VERSION,
                            "instance".to_string(),
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
                Ok(reply) => info!("Created instance: {:#?}", reply),
                Err(err) => error!("Failed to create instance: {}", err),
            };
        }
    };
}
