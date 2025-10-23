use std::time::Duration;

use bevy::{prelude::*, tasks::TaskPool};
use wtransport::{ClientConfig, Endpoint, tls::Sha256Digest};
use xdid::methods::web::reqwest::Url;

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
            match connect_to_world(event).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Error connecting to world server: {e:?}");
                }
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
        world_id: _,
    }: ConnectInfo,
) -> anyhow::Result<()> {
    let cfg = ClientConfig::builder()
        .with_bind_default()
        .with_server_certificate_hashes(vec![cert_hash])
        .max_idle_timeout(Some(Duration::from_mins(1)))?
        .keep_alive_interval(Some(Duration::from_secs(25)))
        .build();

    let endpoint = Endpoint::client(cfg)?;

    let url = url.to_string().replace("http://", "https://");
    info!("Connecting to {url}");

    let connection = endpoint.connect(url).await?;
    info!("Connection opened");

    let _control_stream = connection.open_bi().await?;
    info!("Control stream opened");

    Ok(())
}
