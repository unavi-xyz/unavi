use std::sync::LazyLock;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use bevy::prelude::*;

use bevy_async_task::{AsyncTaskRunner, AsyncTaskStatus};
use dwn::{
    actor::{Actor, MessageBuilder, ProcessMessageError},
    message::{
        descriptor::{records::RecordsFilter, Descriptor},
        Data,
    },
};
use thiserror::Error;
use wired_social::{
    protocols::world_host::{world_host_protocol_url, WORLD_HOST_PROTOCOL_VERSION},
    schemas::{
        common::RecordLink,
        home::{home_schema_url, Home},
        instance::{instance_schema_url, Instance},
        world::{world_schema_url, World},
    },
};

use crate::{InstanceRecord, InstanceServer, UserActor, WorldRecord};

#[derive(Event, Default)]
pub struct JoinHome;

pub fn join_home(mut writer: EventWriter<JoinHome>) {
    writer.send_default();
}

#[derive(Error, Debug)]
pub enum JoinHomeError {
    #[error("Invalid host: {0}")]
    WorldHost(String),
    #[error(transparent)]
    Process(#[from] ProcessMessageError),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Decode(#[from] base64::DecodeError),
}

pub struct JoinHomeResult {
    instance: RecordLink,
    instance_server: String,
    world: RecordLink,
}

const ENV_WORLD_HOST_DID: Option<&str> = option_env!("UNAVI_WORLD_HOST_DID");
const LOCAL_WORLD_HOST_DID: &str = "did:web:localhost%3A3001";

static WORLD_HOST_DID: LazyLock<&str> =
    LazyLock::new(|| ENV_WORLD_HOST_DID.unwrap_or(LOCAL_WORLD_HOST_DID));

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
                let world_host = *WORLD_HOST_DID;

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
                            host: Some(world_host.to_string()),
                            ..default()
                        };

                        let reply = actor
                            .create_record()
                            .data(serde_json::to_vec(&data).unwrap())
                            .data_format("application/json".to_string())
                            .schema(world_schema_url())
                            .published(true)
                            .process()
                            .await?;

                        info!("Created new home world: {}", reply.record_id);

                        // Create home record.
                        let home = Home {
                            world: RecordLink {
                                did: actor.did.clone(),
                                record_id: reply.record_id,
                            },
                        };

                        actor
                            .create_record()
                            .data(serde_json::to_vec(&home).unwrap())
                            .data_format("application/json".to_string())
                            .schema(home_schema_url())
                            .published(true)
                            .process()
                            .await?;

                        home
                    };

                    // Get connect URL.
                    let connect_url_msgs = actor
                        .query_records(RecordsFilter {
                            data_format: Some("text/plain".to_string()),
                            protocol: Some(world_host_protocol_url()),
                            protocol_version: Some(WORLD_HOST_PROTOCOL_VERSION),
                            ..Default::default()
                        })
                        .target(world_host.to_string())
                        .send(world_host)
                        .await?;

                    let connect_url_msg = connect_url_msgs
                        .entries
                        .iter()
                        .find(|m| {
                            if let Descriptor::RecordsWrite(desc) = &m.descriptor {
                                desc.protocol_path == Some("connect-url".to_string())
                            } else {
                                false
                            }
                        })
                        .ok_or(JoinHomeError::WorldHost(
                            "No connect URL found at host".to_string(),
                        ))?;

                    let reply = actor
                        .read_record(connect_url_msg.record_id.clone())
                        .target(world_host.to_string())
                        .send(world_host)
                        .await?;

                    let connect_url = match &reply.record.data {
                        Some(Data::Base64(encoded)) => {
                            let data = URL_SAFE_NO_PAD.decode(encoded)?;
                            String::from_utf8_lossy(&data).to_string()
                        }
                        Some(Data::Encrypted(_)) => {
                            return Err(JoinHomeError::WorldHost(
                                "Host connect URL encrypted. Not currently supported.".to_string(),
                            ))
                        }
                        None => {
                            return Err(JoinHomeError::WorldHost(
                                "No host connect URL found.".to_string(),
                            ))
                        }
                    };

                    // Create instance.
                    let data = Instance {
                        world: home.world.clone(),
                    };

                    let instance_reply = actor
                        .create_record()
                        .protocol(
                            world_host_protocol_url(),
                            WORLD_HOST_PROTOCOL_VERSION,
                            "instance".to_string(),
                        )
                        .data(serde_json::to_vec(&data).unwrap())
                        .data_format("application/json".to_string())
                        .schema(instance_schema_url())
                        .published(true)
                        .target(world_host.to_string())
                        .send(world_host)
                        .await?;

                    info!("Created home instance: {}", instance_reply.record_id);

                    Ok(JoinHomeResult {
                        instance: RecordLink {
                            record_id: instance_reply.record_id,
                            did: world_host.to_string(),
                        },
                        instance_server: connect_url,
                        world: home.world,
                    })
                });
            }
        }
        AsyncTaskStatus::Pending => {}
        AsyncTaskStatus::Finished(res) => {
            match res {
                Ok(JoinHomeResult {
                    instance,
                    instance_server,
                    world,
                }) => {
                    commands.spawn((
                        InstanceRecord(instance),
                        InstanceServer(instance_server),
                        WorldRecord(world),
                    ));
                }
                Err(e) => {
                    error!("{}", e);
                }
            };
        }
    };
}
