use std::{
    collections::HashMap,
    sync::Arc,
    thread::{self, JoinHandle},
};

use bevy::{ecs::entity::Entity, log::*, prelude::Resource};
use scc::HashMap as SccHashMap;
use xdid::core::did_url::DidUrl;

use crate::space::connect_info::ConnectInfo;

use super::{
    connection::host::{HostConnection, connect_to_host},
    streams::publish::{HostStreams, TransformResult},
};

/// Commands sent from Bevy systems to the networking thread.
#[derive(Debug)]
pub enum NetworkCommand {
    Connect {
        entity: Entity,
        info: ConnectInfo,
        space_id: String,
        space_url: DidUrl,
    },
    Disconnect {
        connect_url: String,
    },
    PublishTransform {
        connect_url: String,
        update: TransformResult,
    },
    Shutdown,
}

/// Events sent from the networking thread back to Bevy.
#[derive(Clone)]
pub enum NetworkEvent {
    Connected {
        entity: Entity,
        host: HostConnection,
        connect_url: String,
    },
    ConnectionFailed {
        entity: Entity,
        attempt: u32,
    },
    ConnectionClosed {
        connect_url: String,
    },
}

/// Resource managing the dedicated networking thread.
#[derive(Resource)]
pub struct NetworkingThread {
    pub command_tx: flume::Sender<NetworkCommand>,
    pub event_rx: flume::Receiver<NetworkEvent>,
    _handle: JoinHandle<()>,
}

impl NetworkingThread {
    pub fn spawn() -> Self {
        let (command_tx, command_rx) = flume::unbounded();
        let (event_tx, event_rx) = flume::unbounded();

        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("networking")
                .build()
                .expect("build tokio runtime");

            rt.block_on(async move {
                if let Err(e) = thread_loop(command_rx, event_tx).await {
                    error!("Networking thread error: {e:?}");
                }
            });
        });

        Self {
            command_tx,
            event_rx,
            _handle: handle,
        }
    }
}

/// State tracked by the networking thread.
struct ThreadState {
    /// Active connections by connect_url.
    connections: HashMap<String, ActiveConnection>,
    /// Connection initiation in progress (to prevent duplicates).
    initiating: HashMap<String, Vec<Entity>>,
    /// Transform streams by connect_url.
    streams: Arc<SccHashMap<String, HostStreams>>,
}

struct ActiveConnection {
    host: HostConnection,
    /// Entities (spaces) using this connection.
    users: Vec<Entity>,
}

/// Main loop running on the networking thread.
async fn thread_loop(
    command_rx: flume::Receiver<NetworkCommand>,
    event_tx: flume::Sender<NetworkEvent>,
) -> anyhow::Result<()> {
    let mut state = ThreadState {
        connections: HashMap::new(),
        initiating: HashMap::new(),
        streams: Arc::new(SccHashMap::new()),
    };

    loop {
        match command_rx.recv_async().await? {
            NetworkCommand::Connect {
                entity,
                info,
                space_id,
                space_url,
            } => {
                handle_connect(&mut state, &event_tx, entity, info, space_id, space_url).await;
            }
            NetworkCommand::Disconnect { connect_url } => {
                handle_disconnect(&mut state, &event_tx, &connect_url).await;
            }
            NetworkCommand::PublishTransform {
                connect_url,
                update,
            } => {
                handle_publish(&state, &connect_url, update).await;
            }
            NetworkCommand::Shutdown => {
                info!("Networking thread shutting down");
                break;
            }
        }
    }

    Ok(())
}

async fn handle_connect(
    state: &mut ThreadState,
    event_tx: &flume::Sender<NetworkEvent>,
    entity: Entity,
    info: ConnectInfo,
    space_id: String,
    space_url: DidUrl,
) {
    let connect_url = info.connect_url.to_string();

    // Check if connection already exists.
    if let Some(active) = state.connections.get_mut(&connect_url) {
        active.users.push(entity);
        let _ = event_tx.send(NetworkEvent::Connected {
            entity,
            host: active.host.clone(),
            connect_url,
        });
        return;
    }

    // Check if connection is already being initiated.
    if let Some(waiters) = state.initiating.get_mut(&connect_url) {
        waiters.push(entity);
        return;
    }

    // Start new connection.
    state.initiating.insert(connect_url.clone(), vec![entity]);

    match connect_to_host(info).await {
        Ok(host) => {
            info!("Connected to {connect_url}");

            // Join space session.
            if let Err(e) = join_space(&host, space_id, space_url).await {
                warn!("Failed to join space: {e:?}");
                let _ = event_tx.send(NetworkEvent::ConnectionFailed { entity, attempt: 0 });
                state.initiating.remove(&connect_url);
                return;
            }

            // Spawn stream accept task.
            spawn_stream_accept(&host, connect_url.clone());

            // Add to active connections.
            let mut users = state.initiating.remove(&connect_url).unwrap_or_default();
            users.push(entity);

            state.connections.insert(
                connect_url.clone(),
                ActiveConnection {
                    host: host.clone(),
                    users,
                },
            );

            // Notify all waiting entities.
            let _ = event_tx.send(NetworkEvent::Connected {
                entity,
                host,
                connect_url,
            });
        }
        Err(e) => {
            warn!("Connection failed: {e:?}");
            let _ = event_tx.send(NetworkEvent::ConnectionFailed { entity, attempt: 0 });
            state.initiating.remove(&connect_url);
        }
    }
}

async fn handle_disconnect(
    state: &mut ThreadState,
    event_tx: &flume::Sender<NetworkEvent>,
    connect_url: &str,
) {
    if let Some(active) = state.connections.remove(connect_url) {
        info!("Disconnecting from {connect_url}");
        active
            .host
            .connection
            .close(wtransport::VarInt::from_u32(200), &[]);
        let _ = state.streams.remove_async(connect_url).await;
        let _ = event_tx.send(NetworkEvent::ConnectionClosed {
            connect_url: connect_url.to_string(),
        });
    }
}

async fn handle_publish(state: &ThreadState, connect_url: &str, update: TransformResult) {
    let Some(active) = state.connections.get(connect_url) else {
        return;
    };

    use super::streams::publish::{get_or_create_iframe_stream, send_iframe, send_pframe};

    // Ensure iframe stream exists.
    if let Err(e) =
        get_or_create_iframe_stream(&state.streams, connect_url, &active.host.connection).await
    {
        error!("failed to create iframe stream: {e:?}");
        let _ = state.streams.remove_async(connect_url).await;
        return;
    }

    // Send the transform update.
    let result = match &update {
        TransformResult::IFrame(iframe) => send_iframe(&state.streams, connect_url, iframe).await,
        TransformResult::PFrame(pframe) => {
            send_pframe(&state.streams, connect_url, &active.host.connection, pframe).await
        }
    };

    if let Err(e) = result {
        error!("failed to send transform update: {e:?}");
        let _ = state.streams.remove_async(connect_url).await;
    }
}

async fn join_space(
    host: &HostConnection,
    space_id: String,
    space_url: DidUrl,
) -> anyhow::Result<()> {
    use crate::space::tickrate::{SetTickrate, TICKRATE_QUEUE};
    use std::time::Duration;

    const MIN_TICKRATE: u64 = 25;
    const MAX_TICKRATE: u64 = 1_000;

    let tickrate_ms = host
        .control
        .tickrate_ms(tarpc::context::current())
        .await?
        .clamp(MIN_TICKRATE, MAX_TICKRATE);

    TICKRATE_QUEUE.0.send(SetTickrate {
        space_url,
        tickrate: Duration::from_millis(tickrate_ms),
    })?;

    host.control
        .join_space(tarpc::context::current(), space_id.clone())
        .await?
        .map_err(|e| anyhow::anyhow!("rpc error: {e}"))?;

    info!("Joined space {space_id}");

    Ok(())
}

fn spawn_stream_accept(host: &HostConnection, #[allow(unused_variables)] connect_url: String) {
    let connection = host.connection.clone();
    let transform_channels = host.transform_channels.clone();
    let control_tx = host.control_tx.clone();

    tokio::spawn(async move {
        while let Ok(stream) = connection.accept_uni().await {
            let transform_channels = transform_channels.clone();
            let control_tx = control_tx.clone();

            #[cfg(feature = "devtools-network")]
            let connect_url = connect_url.clone();

            tokio::spawn(async move {
                use crate::space::networking::streams::recv_stream;

                if let Err(e) = recv_stream(
                    stream,
                    transform_channels,
                    control_tx,
                    #[cfg(feature = "devtools-network")]
                    connect_url,
                )
                .await
                {
                    error!("error handling stream: {e:?}");
                };
            });
        }
    });
}
