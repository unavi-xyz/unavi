use std::sync::Arc;

use bevy::{ecs::world::CommandQueue, log::*, prelude::*, tasks::TaskPool};
use scc::{HashMap as SccHashMap, HashSet as SccHashSet};
use wtransport::error::ConnectionError;

use super::{
    host::{HostConnection, connect_to_host},
    session::join_space_session,
    state::{ConnectionAttempt, ConnectionState},
    streams::spawn_stream_accept_task,
};
use crate::{
    async_commands::ASYNC_COMMAND_QUEUE,
    space::{
        Host, HostControlChannel, HostTransformChannels, Space, connect_info::ConnectInfo,
        record_ref_url::parse_record_ref_url,
    },
};

#[derive(Resource, Clone)]
pub struct HostConnections(pub Arc<SccHashMap<String, HostConnection>>);

impl Default for HostConnections {
    fn default() -> Self {
        Self(Arc::new(SccHashMap::new()))
    }
}

#[derive(Resource, Clone)]
pub struct ConnectionInitiated(pub Arc<SccHashSet<String>>);

impl Default for ConnectionInitiated {
    fn default() -> Self {
        Self(Arc::new(SccHashSet::new()))
    }
}

/// Drives the connection state machine for all spaces.
pub fn drive_connection_lifecycle(
    mut commands: Commands,
    connections: Res<HostConnections>,
    initiated: Res<ConnectionInitiated>,
    mut spaces: Query<(
        Entity,
        &Space,
        &ConnectInfo,
        &mut ConnectionState,
        &mut ConnectionAttempt,
    )>,
) {
    for (entity, space, info, mut state, attempt) in spaces.iter_mut() {
        match *state {
            ConnectionState::Disconnected => {
                if attempt.is_ready() {
                    info!("Initiating connection to {}", info.connect_url);
                    *state = ConnectionState::Connecting { attempt: 0 };
                    spawn_connect_task(
                        entity,
                        space,
                        info,
                        &connections.0,
                        &initiated.0,
                        &mut commands,
                    );
                }
            }
            ConnectionState::Connecting { .. } => {}
            ConnectionState::Connected => {}
            ConnectionState::Reconnecting { .. } => {
                if attempt.is_ready() {
                    info!("Retrying connection to {}", info.connect_url);
                    *state = ConnectionState::Connecting {
                        attempt: match *state {
                            ConnectionState::Reconnecting { attempt } => attempt + 1,
                            _ => 0,
                        },
                    };
                    spawn_connect_task(
                        entity,
                        space,
                        info,
                        &connections.0,
                        &initiated.0,
                        &mut commands,
                    );
                }
            }
            ConnectionState::Failed { permanent } => {
                if permanent {
                    commands.entity(entity).remove::<ConnectInfo>();
                }
            }
        }
    }
}

fn spawn_connect_task(
    entity: Entity,
    space: &Space,
    info: &ConnectInfo,
    connections: &Arc<SccHashMap<String, HostConnection>>,
    initiated: &Arc<SccHashSet<String>>,
    commands: &mut Commands,
) {
    let connect_url = info.connect_url.to_string();
    let space_url = space.url.clone();
    let info = info.clone();
    let connections = connections.clone();
    let initiated = initiated.clone();

    let space_id = match parse_record_ref_url(&space.url) {
        Ok(id) => id.to_string(),
        Err(e) => {
            error!("Failed to parse space URL: {e:?}");
            commands
                .entity(entity)
                .insert(ConnectionState::Failed { permanent: true });
            return;
        }
    };

    let pool = TaskPool::get_thread_executor();
    pool.spawn(async move {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build tokio runtime");

        let task = rt.spawn(async move {
            let result: anyhow::Result<(HostConnection, bool)> = async {
                if let Some(host) = connections
                    .read_async(&connect_url, |_, host| host.clone())
                    .await
                {
                    return Ok((host, false));
                }

                if initiated.contains_async(&connect_url).await {
                    return Err(anyhow::anyhow!("connection initiation in progress"));
                }

                let _ = initiated.insert_async(connect_url.clone()).await;
                let result = connect_to_host(info).await;
                let _ = initiated.remove_async(&connect_url).await;

                result.map(|host| (host, true))
            }
            .await;

            match result {
                Ok((host, is_new_connection)) => {
                    handle_connect_success(
                        entity,
                        host.clone(),
                        space_id,
                        space_url,
                        connect_url.clone(),
                        connections.clone(),
                        is_new_connection,
                    )
                    .await;

                    // Keep runtime alive by monitoring connection closure.
                    if is_new_connection {
                        match host.connection.closed().await {
                            ConnectionError::LocallyClosed => {}
                            e => {
                                warn!("Connection closed: {e:?}");
                                let _ = connections.remove_async(&connect_url).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    handle_connect_failure(entity, e).await;
                }
            }
        });

        if let Err(e) = task.await {
            error!("Connection task join error: {e:?}");
        }
    })
    .detach();
}

async fn handle_connect_success(
    entity: Entity,
    host: HostConnection,
    space_id: String,
    space_url: xdid::core::did_url::DidUrl,
    connect_url: String,
    connections: Arc<SccHashMap<String, HostConnection>>,
    is_new_connection: bool,
) {
    info!("Connected to {connect_url}");

    // Per-connection setup.
    if is_new_connection {
        let transform_channels = host.transform_channels.clone();
        let tx = host.control_tx.clone();
        let rx = host.control_rx.clone();
        let connect_url_clone = connect_url.clone();

        let mut queue = CommandQueue::default();
        queue.push(move |world: &mut World| {
            world.spawn((
                Host {
                    connect_url: connect_url_clone,
                },
                HostTransformChannels {
                    players: transform_channels,
                },
                HostControlChannel { tx, rx },
            ));
        });

        if let Err(e) = ASYNC_COMMAND_QUEUE.0.send(queue) {
            error!("failed to spawn host entity: {e:?}");
        }

        spawn_stream_accept_task(
            host.clone(),
            #[cfg(feature = "devtools-network")]
            connect_url.clone(),
        );

        let _ = connections.insert_async(connect_url, host.clone()).await;
    }

    let session_result = join_space_session(&host, space_id, space_url).await;

    match session_result {
        Ok(()) => {
            let mut queue = CommandQueue::default();
            queue.push(move |world: &mut World| {
                if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                    entity_mut.insert(ConnectionState::Connected);
                    if let Some(mut attempt) = entity_mut.get_mut::<ConnectionAttempt>() {
                        attempt.reset();
                    }
                }
            });

            if let Err(e) = ASYNC_COMMAND_QUEUE.0.send(queue) {
                error!("failed to update space state: {e:?}");
            }
        }
        Err(e) => {
            warn!("Failed to join space: {e:?}");
            handle_connect_failure(entity, e).await;
        }
    }
}

async fn handle_connect_failure(entity: Entity, error: anyhow::Error) {
    warn!("Connection failed: {error:?}");

    let mut queue = CommandQueue::default();
    queue.push(move |world: &mut World| {
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            if let Some(mut attempt) = entity_mut.get_mut::<ConnectionAttempt>() {
                attempt.increase_backoff();
            }

            let new_attempt = match entity_mut.get::<ConnectionState>() {
                Some(ConnectionState::Connecting { attempt }) => *attempt + 1,
                _ => 1,
            };

            entity_mut.insert(ConnectionState::Reconnecting {
                attempt: new_attempt,
            });
        }
    });

    if let Err(e) = ASYNC_COMMAND_QUEUE.0.send(queue) {
        error!("failed to update connection state: {e:?}");
    }
}
