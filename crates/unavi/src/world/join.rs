use std::time::Duration;

use bevy::{prelude::*, tasks::TaskPool};
use tarpc::{
    client::Config,
    tokio_serde::formats::Bincode,
    tokio_util::codec::{Framed, LengthDelimitedCodec},
};
use unavi_server_service::ControlServiceClient;
use wtransport::{ClientConfig, Endpoint, stream::BiStream, tls::Sha256Digest};
use xdid::methods::web::reqwest::Url;

use crate::world::WorldConnection;

#[derive(Event)]
pub struct JoinWorld(pub ConnectInfo);

#[derive(Debug, Clone)]
pub struct ConnectInfo {
    pub url: Url,
    pub cert_hash: Sha256Digest,
    pub world_id: String,
}

pub fn handle_join_world(trigger: Trigger<JoinWorld>) {
    let event = trigger.0.clone();

    info!("Joining world: {}@{}", event.world_id, event.url);

    let pool = TaskPool::get_thread_executor();

    pool.spawn(async move {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build tokio runtime");

        let task = rt.spawn(async move {
            let world = match connect_to_world(event).await {
                Ok(w) => w,
                Err(e) => {
                    error!("Error connecting to world server: {e:?}");
                    return;
                }
            };

            if let Err(e) = super::handle_world_connection(world).await {
                error!("Error handling world connection: {e:?}");
            }
        });

        if let Err(e) = task.await {
            error!("Task join error: {e:?}");
        }
    })
    .detach();
}

async fn connect_to_world(
    ConnectInfo {
        url,
        cert_hash,
        world_id,
    }: ConnectInfo,
) -> anyhow::Result<WorldConnection> {
    let cfg = ClientConfig::builder()
        .with_bind_default()
        .with_server_certificate_hashes(vec![cert_hash])
        .max_idle_timeout(Some(Duration::from_mins(1)))?
        .keep_alive_interval(Some(Duration::from_secs(25)))
        .build();

    let endpoint = Endpoint::client(cfg)?;

    let url = url.to_string().replace("http://", "https://");
    let connection = endpoint.connect(url).await?;

    let stream = connection.open_bi().await?.await?;

    let bi_stream = BiStream::join(stream);
    let framed = Framed::new(bi_stream, LengthDelimitedCodec::default());
    let transport = tarpc::serde_transport::new(framed, Bincode::default());

    let control_service = ControlServiceClient::new(Config::default(), transport);

    let control = control_service.spawn();

    control
        .join_world(tarpc::context::current(), world_id.clone())
        .await?
        .map_err(|e| anyhow::anyhow!("rpc error: {e}"))?;

    info!("Joined world {world_id}");

    Ok(WorldConnection {
        connection,
        control,
    })
}
