use anyhow::Result;
use bevy::prelude::*;
use bevy_async_task::{AsyncTaskRunner, AsyncTaskStatus};

const WORLD_ADDRESS: &str = "https://127.0.0.1:3001";

use xwt::current::{RecvStream, SendStream};
#[cfg(target_family = "wasm")]
use xwt_core::{
    traits::{Connecting, EndpointConnect, OpenBiStream, OpeningBiStream},
    Read, Write,
};

pub fn poll_connection_thread(
    mut task_executor: AsyncTaskRunner<Result<()>>,
    mut started: Local<bool>,
) {
    match task_executor.poll() {
        AsyncTaskStatus::Idle => {
            if *started {
                return;
            }

            *started = true;

            task_executor.start(connection_thread());
        }
        AsyncTaskStatus::Pending => {}
        AsyncTaskStatus::Finished(res) => {
            if let Err(e) = res {
                error!("Connection thread error: {}", e);
            }
        }
    }
}

async fn connection_thread() -> Result<()> {
    let (mut send, mut recv) = open_connection(WORLD_ADDRESS).await?;

    send.0.write(b"Hello, world!").await?;

    let mut buf = [0; 10_000];
    let n = recv.0.read(&mut buf).await?.unwrap();
    let msg = std::str::from_utf8(&buf[..n])?;
    info!("Got message from server: {}", msg);

    Ok(())
}

async fn open_connection(addr: &str) -> Result<(SendStream, RecvStream)> {
    #[cfg(not(target_family = "wasm"))]
    let endpoint = {
        let config = wtransport::ClientConfig::builder().with_bind_default();

        #[cfg(not(feature = "disable-cert-validation"))]
        let config = config.with_native_certs();

        #[cfg(feature = "disable-cert-validation")]
        let config = {
            warn!(
                "Certificate validation is disabled. This is not recommended for production use."
            );
            config.with_no_cert_validation()
        };

        xwt::current::Endpoint(wtransport::Endpoint::client(config.build())?).0
    };

    #[cfg(target_family = "wasm")]
    let endpoint = xwt_web_sys::Endpoint::default();

    info!("Connecting to {}", addr);
    let connection = endpoint.connect(addr).await?;

    #[cfg(target_family = "wasm")]
    let connection = connection.wait_connect().await?;

    let opening = connection.open_bi().await?;

    #[cfg(not(target_family = "wasm"))]
    let (send, recv) = opening.await?;

    #[cfg(target_family = "wasm")]
    let (send, recv) = opening.wait_bi().await?;

    Ok((SendStream(send), RecvStream(recv)))
}
