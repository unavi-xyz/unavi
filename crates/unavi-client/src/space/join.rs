use std::time::Duration;

use bevy::{log::tracing::Instrument, prelude::*, tasks::TaskPool};
use tarpc::{
    client::Config,
    tokio_serde::formats::Bincode,
    tokio_util::codec::{Framed, LengthDelimitedCodec},
};
use unavi_server_service::ControlServiceClient;
use wtransport::{ClientConfig, Endpoint, stream::BiStream, tls::Sha256Digest};
use xdid::methods::web::reqwest::Url;

use crate::space::connection::{SpaceConnection, handle_space_connection};

#[derive(Event)]
pub struct JoinSpace(pub ConnectInfo);

#[derive(Debug, Clone)]
pub struct ConnectInfo {
    pub cert_hash: Sha256Digest,
    pub connect_url: Url,
    pub space_id: String,
}

pub fn handle_join_space(trigger: Trigger<JoinSpace>) {
    let event = trigger.0.clone();

    let pool = TaskPool::get_thread_executor();

    pool.spawn(async move {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build tokio runtime");

        let task = rt.spawn(async move {
            let connect_span = info_span!("connect", url = event.connect_url.to_string());

            let space = match connect_to_space(event).instrument(connect_span).await {
                Ok(w) => w,
                Err(e) => {
                    error!("Error connecting to space server: {e:?}");
                    return;
                }
            };

            if let Err(e) = handle_space_connection(space).await {
                error!("Error handling space connection: {e:?}");
            }
        });

        if let Err(e) = task.await {
            error!("Task join error: {e:?}");
        }
    })
    .detach();
}

async fn connect_to_space(
    ConnectInfo {
        connect_url,
        cert_hash,
        space_id,
    }: ConnectInfo,
) -> anyhow::Result<SpaceConnection> {
    let cfg = ClientConfig::builder()
        .with_bind_default()
        .with_server_certificate_hashes(vec![cert_hash])
        .max_idle_timeout(Some(Duration::from_mins(1)))?
        .keep_alive_interval(Some(Duration::from_secs(25)))
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

    control
        .join_space(tarpc::context::current(), space_id.clone())
        .await?
        .map_err(|e| anyhow::anyhow!("rpc error: {e}"))?;

    info!("Joined space {space_id}@{connect_url}");

    Ok(SpaceConnection {
        connection,
        _control: control,
    })
}
