use bevy::prelude::*;
use bevy_async_task::{AsyncTaskRunner, AsyncTaskStatus};
use dwn::{
    actor::{records::WriteResponse, ProcessMessageError},
    message::descriptor::iana_media_types::{Application, MediaType},
};
use wired_protocol::schemas::home::{home_schema_url, Home};

use crate::UserActor;

#[derive(Event)]
pub struct CreateWorld(pub Home);

pub fn handle_create_home(
    actor: Res<UserActor>,
    mut events: EventReader<CreateWorld>,
    mut task: AsyncTaskRunner<Result<WriteResponse, ProcessMessageError>>,
) {
    match task.poll() {
        AsyncTaskStatus::Idle => {
            if let Some(event) = events.read().next() {
                let actor = actor.0.clone();
                let home = event.0.clone();

                task.start(async move {
                    actor
                        .create_record()
                        .data(serde_json::to_vec(&home).unwrap())
                        .data_format(MediaType::Application(Application::Json))
                        .schema(home_schema_url())
                        .published(true)
                        .process()
                        .await
                });
            }
        }
        AsyncTaskStatus::Pending => {}
        AsyncTaskStatus::Finished(res) => {
            match res {
                Ok(reply) => info!("Created home: {:#?}", reply),
                Err(err) => error!("Failed to create home: {}", err),
            };
        }
    };
}
