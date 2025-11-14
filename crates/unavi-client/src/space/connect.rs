use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Duration,
};

use bevy::{log::tracing::Instrument, prelude::*, tasks::TaskPool};
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

use crate::space::{
    Space,
    connect_info::ConnectInfo,
    record_ref_url::parse_record_ref_url,
    tickrate::{SetTickrate, TICKRATE_QUEUE},
};

/// Connection to a host server.
/// Used as transport for one or more spaces.
#[derive(Clone)]
pub struct HostConnection {
    pub connection: Connection,
    pub control: ControlServiceClient,
}

#[derive(Resource, Default)]
pub struct HostConnections(Arc<RwLock<HashMap<String, HostConnection>>>);

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
    let transform_tx = space.transform_tx.clone();

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
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    } else {
                        match connect_to_host(info.clone()).await {
                            Ok(host) => {
                                // Recieve and handle incoming streams.
                                let con = host.connection.clone();
                                let transform_tx = transform_tx.clone();
                                tokio::spawn(async move {
                                    loop {
                                        let Ok(stream) = con.accept_uni().await else {
                                            break;
                                        };

                                        let transform_tx = transform_tx.clone();
                                        tokio::spawn(async move {
                                            if let Err(e) =
                                                super::stream::recv_stream(stream, transform_tx)
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
                                tokio::time::sleep(Duration::from_secs(30)).await;
                                continue;
                            }
                        }
                    };

                    if let Err(e) =
                        handle_space_session(&host, space_id.clone(), space_url.clone()).await
                    {
                        warn!("Space session error: {e:?}");
                        tokio::time::sleep(Duration::from_secs(15)).await;
                        continue;
                    }

                    match host.connection.closed().await {
                        ConnectionError::LocallyClosed => break,
                        e => {
                            warn!("Connection closed: {e:?}");
                            tokio::time::sleep(Duration::from_secs(10)).await;
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
        .max_idle_timeout(Some(Duration::from_mins(1)))?
        .keep_alive_interval(Some(Duration::from_secs(15)))
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

    Ok(HostConnection {
        connection,
        control,
    })
}

const MAX_TICKRATE: u64 = 1_000;
const MIN_TICKRATE: u64 = 25;

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
            if other_spaces_found == 0
                && let Some(connection) = connections.write().await.remove(&connect_url)
            {
                connection.connection.close(VarInt::from_u32(200), &[]);
            }
        });

        if let Err(e) = task.await {
            error!("Task join error: {e:?}");
        }
    })
    .detach();

    Ok(())
}
