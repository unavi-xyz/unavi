use bevy::prelude::*;
use bevy_async_task::{AsyncTaskRunner, AsyncTaskStatus};

const WORLD_ADDRESS: &str = "https://127.0.0.1:3000";

#[cfg(target_family = "wasm")]
use xwt_core::traits::EndpointConnect;

#[cfg(target_family = "wasm")]
use xwt_core::traits::OpenBiStream;

#[cfg(target_family = "wasm")]
use xwt_core::Write;

#[cfg(target_family = "wasm")]
use xwt_core::Read;

pub fn open_connection(mut task_executor: AsyncTaskRunner<String>, mut started: Local<bool>) {
    match task_executor.poll() {
        AsyncTaskStatus::Idle => {
            if *started {
                return;
            }

            *started = true;

            task_executor.start(test_connection_wrapped());
        }
        AsyncTaskStatus::Pending => {}
        AsyncTaskStatus::Finished(v) => {
            println!("Received {v}");
        }
    }
}

async fn test_connection_wrapped() -> String {
    match test_connection().await {
        Ok(v) => v,
        Err(e) => {
            error!("Failed to connect: {}", e);
            String::new()
        }
    }
}

async fn test_connection() -> Result<String, Box<dyn std::error::Error>> {
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

        xwt::current::Endpoint(
            wtransport::Endpoint::client(config.build())
                .expect("Unable to build wtransport client"),
        )
        .0
    };

    #[cfg(target_family = "wasm")]
    let endpoint = xwt_web_sys::Endpoint::default();

    info!("Connecting to {}", WORLD_ADDRESS);

    let connection = endpoint.connect(WORLD_ADDRESS).await?;

    #[cfg(not(target_family = "wasm"))]
    let opening = connection.open_bi().await?;

    #[cfg(target_family = "wasm")]
    let opening = connection.0.open_bi().await?;

    #[cfg(not(target_family = "wasm"))]
    let (mut send, mut recv) = opening.await?;

    #[cfg(target_family = "wasm")]
    let (mut send, mut recv) = opening.0;

    info!("Opened bi stream");

    send.write(b"Hello, world!").await.unwrap();

    let mut buf = [0; 1024];
    let n = recv.read(&mut buf).await.unwrap().unwrap();

    let msg = std::str::from_utf8(&buf[..n]).unwrap();

    Ok(String::from(msg))
}
