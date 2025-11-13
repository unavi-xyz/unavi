use std::time::Duration;

use bevy::{log::tracing::Instrument, prelude::*, tasks::TaskPool};
use tarpc::{
    client::Config,
    tokio_serde::formats::Bincode,
    tokio_util::codec::{Framed, LengthDelimitedCodec},
};
use unavi_server_service::ControlServiceClient;
use wtransport::{ClientConfig, Endpoint, stream::BiStream};

use crate::{
    networking::{SpaceSession, handle_space_session},
    space::{Space, connect_info::ConnectInfo, record_ref_url::parse_record_ref_url},
};

pub fn handle_space_connect(
    event: On<Add, ConnectInfo>,
    spaces: Query<(&Space, &ConnectInfo)>,
) -> Result {
    let entity = event.entity;

    let Ok((space, info)) = spaces.get(entity) else {
        Err(anyhow::anyhow!("space not found"))?
    };
    let info = info.clone();
    let space_id = parse_record_ref_url(&space.url)?.to_string();
    let space_url = space.url.clone();

    let pool = TaskPool::get_thread_executor();

    pool.spawn(async move {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build tokio runtime");

        let span = info_span!("session", url = info.connect_url.to_string());

        let task = rt
            .spawn(async move {
                let session = match connect_to_space(info).await {
                    Ok(w) => w,
                    Err(e) => {
                        error!("Error connecting to space server: {e:?}");
                        return;
                    }
                };

                if let Err(e) = handle_space_session(session, space_id, space_url).await {
                    error!("Error handling space connection: {e:?}");
                }
            })
            .instrument(span);

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
) -> anyhow::Result<SpaceSession> {
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

    Ok(SpaceSession {
        connection,
        control,
    })
}
