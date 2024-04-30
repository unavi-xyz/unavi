use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bevy::prelude::*;

use bevy_async_task::{AsyncTaskRunner, AsyncTaskStatus};
use dwn::{
    actor::{MessageBuilder, ProcessMessageError},
    message::{
        descriptor::{
            iana_media_types::{Application, MediaType},
            records::RecordsFilter,
        },
        Data,
    },
};
use thiserror::Error;
use unavi_dwn::{registry::registry_did, UserActor};
use wired_protocol::{
    protocols::world_registry::{registry_protocol_url, REGISTRY_PROTOCOL_VERSION},
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
                            host: Some(registry_did().to_string()),
                            ..default()
                        };

                        let reply = actor
                            .create_record()
                            .data_format(MediaType::Application(Application::Json))
                            .data(serde_json::to_vec(&data).unwrap())
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
                            .data_format(MediaType::Application(Application::Json))
                            .data(serde_json::to_vec(&home).unwrap())
                            .schema(home_schema_url())
                            .process()
                            .await?;

                        home
                    };

                    // Create instance.
                    let data = Instance {
                        world: home.world.clone(),
                    };
                    let registry = registry_did();

                    let reply = actor
                        .create_record()
                        .protocol(
                            registry_protocol_url(),
                            REGISTRY_PROTOCOL_VERSION,
                            "instance".to_string(),
                        )
                        .data(serde_json::to_vec(&data).unwrap())
                        .data_format(MediaType::Application(Application::Json))
                        .schema(instance_schema_url())
                        .target(registry.to_string())
                        .send(registry)
                        .await?;

                    info!("Created home instance: {}", reply.record_id);

                    Ok(JoinHomeResult {
                        instance: RecordLink {
                            record: reply.record_id,
                            did: registry.to_string(),
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
                    error!("Failed to query record: {}", err);
                }
            };
        }
    };
}
