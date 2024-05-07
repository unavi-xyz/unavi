use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bevy::prelude::*;

use bevy_async_task::{AsyncTaskRunner, AsyncTaskStatus};
use dwn::{
    actor::{MessageBuilder, ProcessMessageError},
    message::{
        descriptor::{iana_media_types::Application, records::RecordsFilter},
        Data,
    },
};
use thiserror::Error;
use unavi_dwn::{world_host::world_host_did, UserActor};
use wired_protocol::{
    protocols::world_host::{world_host_protocol_url, WORLD_HOST_PROTOCOL_VERSION},
    schemas::{
        common::RecordLink,
        home::{home_schema_url, Home},
        instance::{instance_schema_url, Instance},
        world::{world_schema_url, World},
    },
};

use crate::{WorldInstance, WorldRecord};

#[derive(Event, Default)]
pub struct JoinHome;

pub fn join_home(mut writer: EventWriter<JoinHome>) {
    writer.send_default();
}

#[derive(Error, Debug)]
pub enum JoinHomeError {
    #[error(transparent)]
    Process(#[from] ProcessMessageError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Decode(#[from] base64::DecodeError),
}

pub struct JoinHomeResult {
    instance: RecordLink,
    world: RecordLink,
}

pub fn handle_join_home(
    actor: Res<UserActor>,
    mut commands: Commands,
    mut events: EventReader<JoinHome>,
    mut task: AsyncTaskRunner<Result<JoinHomeResult, JoinHomeError>>,
) {
    match task.poll() {
        AsyncTaskStatus::Idle => {
            if events.read().next().is_some() {
                let actor = actor.0.clone();

                task.start(async move {
                    let world_host = world_host_did();

                    // Query for user's home.
                    let reply = actor
                        .query_records(RecordsFilter {
                            schema: Some(home_schema_url()),
                            ..default()
                        })
                        .process()
                        .await?;

                    let home = if let Some(msg) = reply.entries.first() {
                        // Query world.
                        let data = match &msg.data {
                            Some(Data::Base64(value)) => URL_SAFE_NO_PAD.decode(value)?,
                            Some(Data::Encrypted(_data)) => {
                                todo!("Home world data encrypted")
                            }
                            None => {
                                todo!("Home world not found")
                            }
                        };

                        let home: Home = serde_json::from_slice(&data)?;

                        home
                    } else {
                        // Create new world.
                        let data = World {
                            name: Some("Home".to_string()),
                            host: Some(world_host.to_string()),
                            ..default()
                        };

                        let reply = actor
                            .create_record()
                            .data(serde_json::to_vec(&data).unwrap())
                            .data_format(Application::Json.into())
                            .schema(world_schema_url())
                            .process()
                            .await?;

                        info!("Created new home world: {}", reply.record_id);

                        // Create home record.
                        let home = Home {
                            world: RecordLink {
                                did: actor.did.clone(),
                                record: reply.record_id,
                            },
                        };

                        actor
                            .create_record()
                            .data(serde_json::to_vec(&home).unwrap())
                            .data_format(Application::Json.into())
                            .schema(home_schema_url())
                            .process()
                            .await?;

                        home
                    };

                    // Create instance.
                    let data = Instance {
                        world: home.world.clone(),
                    };

                    let reply = actor
                        .create_record()
                        .protocol(
                            world_host_protocol_url(),
                            WORLD_HOST_PROTOCOL_VERSION,
                            "instance".to_string(),
                        )
                        .data(serde_json::to_vec(&data).unwrap())
                        .data_format(Application::Json.into())
                        .schema(instance_schema_url())
                        .target(world_host.to_string())
                        .send(world_host)
                        .await?;

                    info!("Created home instance: {}", reply.record_id);

                    Ok(JoinHomeResult {
                        instance: RecordLink {
                            record: reply.record_id,
                            did: world_host.to_string(),
                        },
                        world: home.world,
                    })
                });
            }
        }
        AsyncTaskStatus::Pending => {}
        AsyncTaskStatus::Finished(res) => {
            match res {
                Ok(JoinHomeResult { instance, world }) => {
                    commands.spawn((WorldInstance(instance), WorldRecord(world)));
                }
                Err(err) => {
                    error!("Failed to join home: {}", err);
                }
            };
        }
    };
}
