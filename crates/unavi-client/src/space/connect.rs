use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::Sender,
    sync::{Arc, Mutex},
    time::Duration,
};

use bevy::{ecs::world::CommandQueue, log::tracing::Instrument, prelude::*, tasks::TaskPool};
use tarpc::{
    client::Config,
    tokio_serde::formats::Bincode,
    tokio_util::codec::{Framed, LengthDelimitedCodec},
};
use tokio::{sync::RwLock, task::AbortHandle};
use unavi_server_service::ControlServiceClient;
use wtransport::{
    ClientConfig, Connection, Endpoint, VarInt, error::ConnectionError, stream::BiStream,
};
use xdid::core::did_url::DidUrl;

use crate::{
    async_commands::ASYNC_COMMAND_QUEUE,
    space::{
        Host, HostPlayers, HostTransformChannel, Space,
        connect_info::ConnectInfo,
        record_ref_url::parse_record_ref_url,
        streams::publish::HostTransformStreams,
        tickrate::{SetTickrate, TICKRATE_QUEUE},
    },
};

const MAX_IDLE_TIMEOUT: Duration = Duration::from_mins(2);
const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(15);

const INITIAL_RETRY_DELAY: Duration = Duration::from_secs(1);
const CONNECTION_RETRY_DELAY: Duration = Duration::from_secs(10);
const SESSION_RETRY_DELAY: Duration = Duration::from_secs(15);
const SPACE_CONNECT_RETRY_DELAY: Duration = Duration::from_secs(30);

const MIN_TICKRATE: u64 = 25;
const MAX_TICKRATE: u64 = 1_000;

/// Connection to a host server.
/// Used as transport for one or more spaces.
#[derive(Clone)]
pub struct HostConnection {
    pub connection: Connection,
    pub control: ControlServiceClient,
    pub transform_tx: Sender<crate::space::streams::transform::RecievedTransform>,
}

#[derive(Resource, Default)]
pub struct HostConnections(pub Arc<RwLock<HashMap<String, HostConnection>>>);

#[derive(Resource, Default)]
pub struct SpaceSessions(Arc<RwLock<HashMap<String, AbortHandle>>>);

pub fn handle_space_connect(
    event: On<Add, ConnectInfo>,
    spaces: Query<(&Space, &ConnectInfo)>,
    connections: Res<HostConnections>,
    sessions: Res<SpaceSessions>,
    initiated: Local<Arc<RwLock<HashSet<String>>>>,
) -> Result {
    let Ok((space, info)) = spaces.get(event.entity) else {
        Err(anyhow::anyhow!("space not found"))?
    };
    let info = info.clone();
    let connect_url = info.connect_url.to_string();
    let space_id = parse_record_ref_url(&space.url)?.to_string();
    let space_url = space.url.clone();

    let connections = connections.0.clone();
    let initiated = initiated.clone();
    let sessions = sessions.0.clone();

    let pool = TaskPool::get_thread_executor();
    pool.spawn(async move {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build tokio runtime");

        let span = info_span!("connection", url = connect_url);
        let space_url_str = space_url.to_string();

        let task = rt.spawn(
            async move {
                loop {
                    let host = if let Some(host) = connections.read().await.get(&connect_url) {
                        host.clone()
                    } else if initiated.read().await.contains(&connect_url) {
                        tokio::time::sleep(INITIAL_RETRY_DELAY).await;
                        continue;
                    } else {
                        match connect_to_host(info.clone()).await {
                            Ok(host) => {
                                // Recieve and handle incoming streams.
                                let con = host.connection.clone();
                                let transform_tx = host.transform_tx.clone();
                                tokio::spawn(async move {
                                    loop {
                                        let Ok(stream) = con.accept_uni().await else {
                                            break;
                                        };

                                        let transform_tx = transform_tx.clone();
                                        tokio::spawn(async move {
                                            if let Err(e) =
                                                super::streams::recv_stream(stream, transform_tx)
                                                    .await
                                            {
                                                error!("Error handling stream: {e:?}");
                                            };
                                        });
                                    }
                                });

                                // Save the connection for re-use by other spaces.
                                let mut initiated = initiated.write().await;
                                let mut connections = connections.write().await;
                                initiated.remove(&connect_url);
                                connections.insert(connect_url.clone(), host.clone());

                                host
                            }
                            Err(e) => {
                                warn!("Failed to connect to space: {e:?}");
                                tokio::time::sleep(SPACE_CONNECT_RETRY_DELAY).await;
                                continue;
                            }
                        }
                    };

                    if let Err(e) =
                        handle_space_session(&host, space_id.clone(), space_url.clone()).await
                    {
                        warn!("Space session error: {e:?}");
                        tokio::time::sleep(SESSION_RETRY_DELAY).await;
                        continue;
                    }

                    match host.connection.closed().await {
                        ConnectionError::LocallyClosed => break,
                        e => {
                            warn!("Connection closed: {e:?}");
                            tokio::time::sleep(CONNECTION_RETRY_DELAY).await;
                        }
                    }
                }
            }
            .instrument(span),
        );

        let session_handle = task.abort_handle();
        sessions.write().await.insert(space_url_str, session_handle);

        if let Err(e) = task.await {
            error!("Task join error: {e:?}");
        }
    })
    .detach();

    Ok(())
}

async fn connect_to_host(
    ConnectInfo {
        connect_url,
        cert_hash,
    }: ConnectInfo,
) -> anyhow::Result<HostConnection> {
    let cfg = ClientConfig::builder()
        .with_bind_default()
        .with_server_certificate_hashes(vec![cert_hash])
        .max_idle_timeout(Some(MAX_IDLE_TIMEOUT))?
        .keep_alive_interval(Some(KEEP_ALIVE_INTERVAL))
        .build();

    let endpoint = Endpoint::client(cfg)?;

    let url = connect_url.to_string().replace("http://", "https://");
    let connection = endpoint.connect(url).await?;

    let stream = connection.open_bi().await?.await?;

    let bi_stream = BiStream::join(stream);
    let framed = Framed::new(bi_stream, LengthDelimitedCodec::default());
    let transport = tarpc::serde_transport::new(framed, Bincode::default());

    let control_service = ControlServiceClient::new(Config::default(), transport);

    let control = control_service.spawn();

    // Spawn Host entity.
    let (transform_tx, transform_rx) = std::sync::mpsc::channel();
    let connect_url_clone = connect_url.to_string();

    let transform_tx_clone = transform_tx.clone();
    let mut queue = CommandQueue::default();
    queue.push(move |world: &mut World| {
        world.spawn((
            Host {
                connect_url: connect_url_clone,
            },
            HostTransformChannel {
                tx: transform_tx_clone,
                rx: Arc::new(Mutex::new(transform_rx)),
            },
        ));
    });

    ASYNC_COMMAND_QUEUE.0.send(queue)?;

    Ok(HostConnection {
        connection,
        control,
        transform_tx,
    })
}

async fn handle_space_session(
    host: &HostConnection,
    space_id: String,
    space_url: DidUrl,
) -> anyhow::Result<()> {
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

pub fn handle_space_disconnect(
    event: On<Remove, ConnectInfo>,
    connections: Res<HostConnections>,
    sessions: Res<SpaceSessions>,
    transform_streams: Res<HostTransformStreams>,
    spaces: Query<(Entity, &Space, &ConnectInfo), With<Space>>,
) -> Result {
    let Ok((_, space, info)) = spaces.get(event.entity) else {
        Err(anyhow::anyhow!("space not found"))?
    };

    let mut other_spaces_found = 0;

    for (other_entity, _, other_info) in spaces.iter() {
        if other_entity == event.entity {
            continue;
        }
        if other_info.connect_url != info.connect_url {
            continue;
        }
        other_spaces_found += 1;
    }

    let connect_url = info.connect_url.to_string();
    let connections = connections.0.clone();
    let sessions = sessions.0.clone();
    let streams = transform_streams.0.clone();
    let space_url = space.url.to_string();

    let pool = TaskPool::get_thread_executor();
    pool.spawn(async move {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build tokio runtime");

        let task = rt.spawn(async move {
            // End space session.
            if let Some(handle) = sessions.write().await.remove(&space_url) {
                handle.abort();
            }

            // Disconnect from host, if no other spaces using the connection.
            if other_spaces_found == 0 {
                streams.lock().await.remove(&connect_url);

                if let Some(connection) = connections.write().await.remove(&connect_url) {
                    connection.connection.close(VarInt::from_u32(200), &[]);

                    // Despawn Host entity and all remote players.
                    let connect_url_clone = connect_url.clone();
                    let mut queue = CommandQueue::default();
                    queue.push(move |world: &mut World| {
                        // Find Host entity by connect_url.
                        let mut host_entity = None;
                        let mut query = world.query::<(Entity, &Host)>();
                        for (entity, host) in query.iter(world) {
                            if host.connect_url == connect_url_clone {
                                host_entity = Some(entity);
                                break;
                            }
                        }

                        if let Some(host_ent) = host_entity {
                            // Get all remote players.
                            if let Some(host_players) = world.get::<HostPlayers>(host_ent) {
                                let players = host_players.0.clone();
                                for player in players {
                                    world.despawn(player);
                                }
                            }

                            // Despawn host.
                            world.despawn(host_ent);
                        }
                    });

                    if let Err(e) = ASYNC_COMMAND_QUEUE.0.send(queue) {
                        error!("Failed to send cleanup commands: {e:?}");
                    }
                }
            }
        });

        if let Err(e) = task.await {
            error!("Task join error: {e:?}");
        }
    })
    .detach();

    Ok(())
}
