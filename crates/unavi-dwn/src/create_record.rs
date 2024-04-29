use bevy::prelude::*;
use bevy_async_task::{AsyncTaskRunner, AsyncTaskStatus};
use dwn::{
    actor::{records::WriteResponse, ProcessMessageError},
    message::descriptor::iana_media_types::MediaType,
};

use crate::UserActor;

#[derive(Component)]
pub struct CreateRecord {
    pub record_id: Option<String>,
    pub data: Vec<u8>,
    pub data_format: MediaType,
    pub schema: Option<String>,
    pub published: bool,
}

#[derive(Component)]
pub struct CreateRecordResult(pub WriteResponse);

pub fn handle_create_record(
    actor: Res<UserActor>,
    creates: Query<(Entity, &CreateRecord)>,
    mut commands: Commands,
    mut processing: Local<Option<Entity>>,
    mut task: AsyncTaskRunner<Result<WriteResponse, ProcessMessageError>>,
) {
    match task.poll() {
        AsyncTaskStatus::Idle => {
            if let Some((entity, create)) = creates.iter().next() {
                let actor = actor.0.clone();

                let data = create.data.clone();
                let data_format = create.data_format.clone();
                let published = create.published;
                let schema = create.schema.clone();

                task.start(async move {
                    let mut msg = actor
                        .create_record()
                        .data(data)
                        .data_format(data_format)
                        .published(published);

                    if let Some(schema) = schema {
                        msg = msg.schema(schema.to_owned());
                    }

                    msg.process().await
                });

                *processing = Some(entity);
            }
        }
        AsyncTaskStatus::Pending => {}
        AsyncTaskStatus::Finished(res) => {
            match res {
                Ok(reply) => {
                    commands
                        .entity(processing.unwrap())
                        .insert(CreateRecordResult(reply));
                }
                Err(err) => {
                    error!("Failed to create record: {}", err);
                }
            };

            *processing = None;
        }
    };
}
