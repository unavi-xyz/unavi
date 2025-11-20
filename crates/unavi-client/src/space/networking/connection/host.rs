use std::time::Duration;

use scc::HashMap as SccHashMap;
use tarpc::{
    client::Config,
    tokio_serde::formats::Bincode,
    tokio_util::codec::{Framed, LengthDelimitedCodec},
};
use unavi_server_service::{ControlServiceClient, from_server::ControlMessage};
use wtransport::{ClientConfig, Connection, Endpoint, stream::BiStream, tls::Sha256Digest};
use xdid::methods::web::reqwest::Url;

use crate::space::{connect_info::ConnectInfo, networking::streams::TransformChannels};

const MAX_IDLE_TIMEOUT: Duration = Duration::from_secs(60);
const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(15);

#[derive(Clone)]
pub struct HostConnection {
    pub connection: Connection,
    pub control: ControlServiceClient,
    pub transform_channels: TransformChannels,
    pub control_tx: flume::Sender<ControlMessage>,
    pub control_rx: flume::Receiver<ControlMessage>,
}

/// Connects to a host server and sets up the control RPC channel.
pub async fn connect_to_host(
    ConnectInfo {
        connect_url,
        cert_hash,
    }: ConnectInfo,
) -> anyhow::Result<HostConnection> {
    let connection = create_connection(connect_url.clone(), cert_hash).await?;
    let control = create_control_client(&connection).await?;
    let (control_tx, control_rx) = flume::bounded(64);
    let transform_channels = std::sync::Arc::new(SccHashMap::new());

    Ok(HostConnection {
        connection,
        control,
        transform_channels,
        control_tx,
        control_rx,
    })
}

async fn create_connection(
    connect_url: Url,
    cert_hash: Sha256Digest,
) -> anyhow::Result<Connection> {
    let cfg = ClientConfig::builder()
        .with_bind_default()
        .with_server_certificate_hashes(vec![cert_hash])
        .max_idle_timeout(Some(MAX_IDLE_TIMEOUT))?
        .keep_alive_interval(Some(KEEP_ALIVE_INTERVAL))
        .build();

    let endpoint = Endpoint::client(cfg)?;
    let url = connect_url.to_string().replace("http://", "https://");
    let connection = endpoint.connect(url).await?;

    Ok(connection)
}

async fn create_control_client(connection: &Connection) -> anyhow::Result<ControlServiceClient> {
    let stream = connection.open_bi().await?.await?;
    let bi_stream = BiStream::join(stream);
    let framed = Framed::new(bi_stream, LengthDelimitedCodec::default());
    let transport = tarpc::serde_transport::new(framed, Bincode::default());
    let control_service = ControlServiceClient::new(Config::default(), transport);
    let control = control_service.spawn();

    Ok(control)
}
