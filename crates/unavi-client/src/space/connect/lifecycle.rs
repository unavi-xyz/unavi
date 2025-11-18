use std::sync::Arc;

use bevy::{ecs::world::CommandQueue, log::*, prelude::*, tasks::TaskPool};
use scc::{HashMap as SccHashMap, HashSet as SccHashSet};
use wtransport::error::ConnectionError;

use super::{
    host::{HostConnection, connect_to_host},
    session::join_space_session,
    state::{ConnectionAttempt, ConnectionState, ConnectionTasks},
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
        &mut ConnectionTasks,
    )>,
) {
    for (entity, space, info, mut state, attempt, mut tasks) in spaces.iter_mut() {
        match *state {
            ConnectionState::Disconnected => {
                // Ready to connect.
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
            ConnectionState::Connecting { .. } => {
                // Task is running, state will be updated by task completion.
            }
            ConnectionState::Connected => {
                // Monitor connection health in separate system.
            }
            ConnectionState::Reconnecting { .. } => {
                // Wait for backoff timer.
                if attempt.is_ready() {
                    info!("Retrying connection to {}", info.connect_url);
                    tasks.abort_all();
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
                    // Permanent failure, remove connection info to stop retries.
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
            let result = async {
                // Check if connection already exists.
                if let Some(host) = connections
                    .read_async(&connect_url, |_, host| host.clone())
                    .await
                {
                    return Ok(host);
                }

                // Check if another task is initiating this connection.
                if initiated.contains_async(&connect_url).await {
                    return Err(anyhow::anyhow!("Connection initiation in progress"));
                }

                // Mark as initiated.
                let _ = initiated.insert_async(connect_url.clone()).await;

                // Attempt connection.
                let result = connect_to_host(info).await;

                // Clear initiated flag.
                let _ = initiated.remove_async(&connect_url).await;

                result
            }
            .await;

            match result {
                Ok(host) => {
                    handle_connect_success(
                        entity,
                        host,
                        space_id,
                        space_url,
                        connect_url,
                        connections,
                    )
                    .await;
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
) {
    info!("Connected to {connect_url}");

    // Spawn stream accept task.
    let stream_handle = spawn_stream_accept_task(
        host.clone(),
        #[cfg(feature = "devtools-network")]
        connect_url.clone(),
    );

    // Join space session.
    let session_result = join_space_session(&host, space_id, space_url).await;

    match session_result {
        Ok(()) => {
            // Spawn Host entity.
            let transform_channels = host.transform_channels.clone();
            let connect_url_clone = connect_url.clone();

            let (control_tx_ecs, control_rx) = std::sync::mpsc::sync_channel(16);
            let mut queue = CommandQueue::default();
            queue.push(move |world: &mut World| {
                world.spawn((
                    Host {
                        connect_url: connect_url_clone,
                    },
                    HostTransformChannels {
                        players: transform_channels,
                    },
                    HostControlChannel {
                        tx: control_tx_ecs,
                        rx: Arc::new(std::sync::Mutex::new(control_rx)),
                    },
                ));

                // Update space entity state.
                if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                    entity_mut.insert((
                        ConnectionState::Connected,
                        ConnectionTasks {
                            session_handle: None,
                            stream_accept_handle: Some(stream_handle),
                        },
                    ));
                    if let Some(mut attempt) = entity_mut.get_mut::<ConnectionAttempt>() {
                        attempt.reset();
                    }
                }
            });

            if let Err(e) = ASYNC_COMMAND_QUEUE.0.send(queue) {
                error!("Failed to send commands: {e:?}");
            }

            // Save connection for reuse.
            let _ = connections.insert_async(connect_url, host).await;
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
        error!("Failed to send commands: {e:?}");
    }
}

/// Monitors connected spaces for connection closure.
pub fn check_connection_health(
    connections: Res<HostConnections>,
    spaces: Query<(&ConnectInfo, &ConnectionState)>,
) {
    let pool = TaskPool::get_thread_executor();

    for (info, state) in spaces.iter() {
        if !matches!(state, ConnectionState::Connected) {
            continue;
        }

        let connect_url = info.connect_url.to_string();
        let connections = connections.0.clone();

        pool.spawn(async move {
            if let Some(host) = connections
                .read_async(&connect_url, |_, host| host.clone())
                .await
            {
                match host.connection.closed().await {
                    ConnectionError::LocallyClosed => {
                        // Intentional disconnect, don't reconnect.
                    }
                    e => {
                        warn!("Connection closed: {e:?}");
                        // Remove connection to trigger reconnect.
                        let _ = connections.remove_async(&connect_url).await;
                    }
                }
            }
        })
        .detach();
    }
}
