use std::{net::UdpSocket, time::Duration};

use tracing::info;
use tracing_test::traced_test;
use unavi_server::ServerOptions;
use wtransport::{ClientConfig, Endpoint};

#[tokio::test]
#[traced_test]
async fn test_connect_wtransport() {
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let addr = socket.local_addr().unwrap();
    let port = addr.port();
    drop(socket);

    tokio::task::spawn(async move {
        unavi_server::run_server(ServerOptions {
            port,
            in_memory: true,
        })
        .await
        .unwrap();
    });

    tokio::time::sleep(Duration::from_secs(2)).await;

    let cfg = ClientConfig::builder()
        .with_bind_default()
        .with_no_cert_validation()
        .max_idle_timeout(Some(Duration::from_secs(3)))
        .unwrap()
        .build();
    let endpoint = Endpoint::client(cfg).unwrap();

    let url = format!("https://localhost:{port}");
    info!("connecting to {url}");
    let _conn = endpoint.connect(url).await.unwrap();
}
