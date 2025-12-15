use std::{
    collections::HashMap,
    str::FromStr,
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::Context;
use bevy::{
    ecs::entity::Entity,
    log::{error, info, warn},
    prelude::Resource,
};
use dwn::Actor;
use scc::HashMap as SccHashMap;
use serde::Deserialize;
use unavi_constants::WP_VERSION;
use wired_protocol::HOST_PROTOCOL;
use wtransport::tls::Sha256Digest;
use xdid::{
    core::{did::Did, did_url::DidUrl},
    methods::web::reqwest::Url,
};

use super::{
    connection::host::{HostConnection, connect_to_host},
    streams::publish::{HostStreams, TransformResult},
};

#[derive(Clone, Debug)]
pub(super) struct ConnectInfo {
    pub connect_url: Url,
    pub cert_hash: Sha256Digest,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParsedConnectInfo {
    url: String,
    cert_hash: String,
}

/// Commands sent from Bevy systems to the networking thread.
pub enum NetworkCommand {
    SetActor { actor: Actor },
    JoinSpace { entity: Entity, space_url: DidUrl },
    LeaveSpace { entity: Entity },
    PublishTransform { update: TransformResult },
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
    /// DWN actor for fetching connect info.
    actor: Option<Actor>,
    /// Active connections by `connect_url`.
    connections: HashMap<String, ActiveConnection>,
    /// Connection initiation in progress (to prevent duplicates).
    initiating: HashMap<String, Vec<Entity>>,
    /// Transform streams by `connect_url`.
    streams: Arc<SccHashMap<String, HostStreams>>,
    /// Mapping from space entity to `connect_url`.
    entity_to_url: HashMap<Entity, String>,
}

struct ActiveConnection {
    host: HostConnection,
    /// Entities (spaces) using this connection.
    space_entities: Vec<Entity>,
}

/// Main loop running on the networking thread.
async fn thread_loop(
    command_rx: flume::Receiver<NetworkCommand>,
    event_tx: flume::Sender<NetworkEvent>,
) -> anyhow::Result<()> {
    let mut state = ThreadState {
        actor: None,
        connections: HashMap::new(),
        initiating: HashMap::new(),
        streams: Arc::new(SccHashMap::new()),
        entity_to_url: HashMap::new(),
    };

    loop {
        match command_rx.recv_async().await? {
            NetworkCommand::SetActor { actor } => {
                state.actor = Some(actor);
            }
            NetworkCommand::JoinSpace { entity, space_url } => {
                handle_join_space(&mut state, &event_tx, entity, space_url).await;
            }
            NetworkCommand::LeaveSpace { entity } => {
                handle_leave_space(&mut state, &event_tx, entity).await;
            }
            NetworkCommand::PublishTransform { update } => {
                handle_publish(&state, update).await;
            }
            NetworkCommand::Shutdown => {
                info!(
                    "Networking thread shutting down - closing {} connections",
                    state.connections.len()
                );

                // Close all active connections gracefully.
                for (connect_url, active) in state.connections.drain() {
                    info!("Closing connection to {connect_url}");
                    active
                        .host
                        .connection
                        .close(wtransport::VarInt::from_u32(200), b"shutdown");
                }

                info!("Shutdown complete");

                // Wait for connections to finish.
                tokio::time::sleep(Duration::from_secs(1)).await;

                break;
            }
        }
    }

    Ok(())
}

async fn fetch_connect_info(
    actor: &Actor,
    host_did: &Did,
    host_dwn: &Url,
) -> anyhow::Result<Option<ConnectInfo>> {
    let found_urls = actor
        .query()
        .protocol(HOST_PROTOCOL.to_string())
        .protocol_version(WP_VERSION)
        .protocol_path("server".to_string())
        .target(host_did)
        .send(host_dwn)
        .await
        .context("query connect url")?;

    for found in found_urls {
        let record_id = found.entry().record_id.clone();

        let data = if let Some(d) = found.into_data() {
            d
        } else {
            let Some(read) = actor
                .read(record_id)
                .target(host_did)
                .send(host_dwn)
                .await?
            else {
                warn!("connect url record not found");
                continue;
            };
            let Some(data) = read.into_data() else {
                warn!("connect url data not found");
                continue;
            };
            data
        };

        let parsed: ParsedConnectInfo = serde_json::from_slice(&data)?;

        let connect_url = Url::parse(&parsed.url)?;
        let cert_hash = Sha256Digest::from_str(&parsed.cert_hash)?;

        return Ok(Some(ConnectInfo {
            connect_url,
            cert_hash,
        }));
    }

    Ok(None)
}

async fn handle_join_space(
    state: &mut ThreadState,
    event_tx: &flume::Sender<NetworkEvent>,
    entity: Entity,
    space_url: DidUrl,
) {
    use crate::space::record_ref_url::parse_record_ref_url;

    // Parse space ID from URL.
    let space_id = match parse_record_ref_url(&space_url) {
        Ok(id) => id.to_string(),
        Err(e) => {
            error!("Failed to parse space URL {space_url}: {e:?}");
            let _ = event_tx.send(NetworkEvent::ConnectionFailed { entity, attempt: 0 });
            return;
        }
    };

    // Get actor.
    let Some(actor) = &state.actor else {
        warn!("Actor not set, cannot join space");
        let _ = event_tx.send(NetworkEvent::ConnectionFailed { entity, attempt: 0 });
        return;
    };

    // Fetch connect info from DWN.
    let host_did = &space_url.did;
    let Some(host_dwn) = actor.remote.clone() else {
        error!("No remote DWN configured");
        let _ = event_tx.send(NetworkEvent::ConnectionFailed { entity, attempt: 0 });
        return;
    };

    let info = match fetch_connect_info(actor, host_did, &host_dwn).await {
        Ok(Some(info)) => info,
        Ok(None) => {
            warn!("Connect info not found for space {space_url}");
            let _ = event_tx.send(NetworkEvent::ConnectionFailed { entity, attempt: 0 });
            return;
        }
        Err(e) => {
            error!("Failed to fetch connect info: {e:?}");
            let _ = event_tx.send(NetworkEvent::ConnectionFailed { entity, attempt: 0 });
            return;
        }
    };

    let connect_url = info.connect_url.to_string();

    // Check if connection already exists.
    if let Some(active) = state.connections.get_mut(&connect_url) {
        active.space_entities.push(entity);
        state.entity_to_url.insert(entity, connect_url.clone());
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
            if let Err(e) = join_space(&host, space_id, space_url.clone()).await {
                warn!("Failed to join space: {e:?}");
                let _ = event_tx.send(NetworkEvent::ConnectionFailed { entity, attempt: 0 });
                state.initiating.remove(&connect_url);
                return;
            }

            // Spawn stream accept task.
            spawn_stream_accept(
                &host,
                #[cfg(feature = "devtools-network")]
                connect_url.clone(),
            );

            // Add to active connections.
            let space_entities = state.initiating.remove(&connect_url).unwrap_or_default();

            for &ent in &space_entities {
                state.entity_to_url.insert(ent, connect_url.clone());
            }

            state.connections.insert(
                connect_url.clone(),
                ActiveConnection {
                    host: host.clone(),
                    space_entities,
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

async fn handle_leave_space(
    state: &mut ThreadState,
    event_tx: &flume::Sender<NetworkEvent>,
    entity: Entity,
) {
    // Remove entity->url mapping.
    let Some(connect_url) = state.entity_to_url.remove(&entity) else {
        return;
    };

    // Remove entity from connection's space list.
    let should_disconnect = if let Some(active) = state.connections.get_mut(&connect_url) {
        active.space_entities.retain(|&e| e != entity);
        active.space_entities.is_empty()
    } else {
        false
    };

    // Disconnect if no more spaces using this connection.
    if should_disconnect && let Some(active) = state.connections.remove(&connect_url) {
        info!("Disconnecting from {connect_url} (no more spaces)");
        active
            .host
            .connection
            .close(wtransport::VarInt::from_u32(200), &[]);
        let _ = state.streams.remove_async(&connect_url).await;
        let _ = event_tx.send(NetworkEvent::ConnectionClosed {
            connect_url: connect_url.clone(),
        });
    }
}

async fn handle_publish(state: &ThreadState, update: TransformResult) {
    use super::streams::publish::{get_or_create_iframe_stream, send_iframe, send_pframe};

    // Broadcast to all active connections.
    for (connect_url, active) in &state.connections {
        // Ensure iframe stream exists.
        if let Err(e) =
            get_or_create_iframe_stream(&state.streams, connect_url, &active.host.connection).await
        {
            error!("failed to create iframe stream for {connect_url}: {e:?}");
            let _ = state.streams.remove_async(connect_url).await;
            continue;
        }

        // Send the transform update.
        let result = match &update {
            TransformResult::IFrame(iframe) => {
                send_iframe(&state.streams, connect_url, iframe).await
            }
            TransformResult::PFrame(pframe) => {
                send_pframe(&state.streams, connect_url, &active.host.connection, pframe).await
            }
        };

        if let Err(e) = result {
            error!("failed to send transform update to {connect_url}: {e:?}");
            let _ = state.streams.remove_async(connect_url).await;
        }
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

fn spawn_stream_accept(
    host: &HostConnection,
    #[cfg(feature = "devtools-network")] connect_url: String,
) {
    let connection = host.connection.clone();
    let transform_channels = Arc::clone(&host.transform_channels);
    let control_tx = host.control_tx.clone();

    tokio::spawn(async move {
        while let Ok(stream) = connection.accept_uni().await {
            let transform_channels = Arc::clone(&transform_channels);
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
                }
            });
        }
    });
}
