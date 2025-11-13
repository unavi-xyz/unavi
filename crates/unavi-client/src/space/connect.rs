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
use wtransport::{ClientConfig, Connection, Endpoint, VarInt, stream::BiStream};

use crate::{
    networking::handle_space_session,
    space::{Space, connect_info::ConnectInfo, record_ref_url::parse_record_ref_url},
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
                    let connection = if let Some(c) = connections.read().await.get(&connect_url) {
                        c.clone()
                    } else if initiated.read().await.contains(&connect_url) {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        continue;
                    } else {
                        match connect_to_space(info.clone()).await {
                            Ok(c) => {
                                let mut initiated = initiated.write().await;
                                let mut connections = connections.write().await;
                                initiated.remove(&connect_url);
                                connections.insert(connect_url.clone(), c.clone());
                                c
                            }
                            Err(e) => {
                                warn!("Failed to connect to space: {e:?}");
                                tokio::time::sleep(Duration::from_secs(30)).await;
                                continue;
                            }
                        }
                    };

                    if let Err(e) =
                        handle_space_session(connection, space_id.clone(), space_url.clone()).await
                    {
                        warn!("Space session error: {e:?}");
                        tokio::time::sleep(Duration::from_secs(15)).await;
                    }
                }
            }
            .instrument(span),
        );

        let session_handle = task.abort_handle();
        sessions.write().await.insert(space_url_str, session_handle);

        let Err(e) = task.await;
        error!("Task join error: {e:?}");
    })
    .detach();

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

async fn connect_to_space(
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
