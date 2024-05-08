//! Server for running multiplayer instances of worlds over WebTransport.

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::Router;
use tracing::info;

mod cert;
mod connection;
mod world_host;

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub dwn_url: String,
    pub port: u16,
}

pub async fn start(opts: ServerOptions) -> std::io::Result<()> {
    let opts = Arc::new(opts);

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), opts.port);

    // let identity = cert::generate_tls_identity();
    // let options = connection::WorldOptions {
    //     address,
    //     identity: &identity,
    // };

    // connection::start_server(options)
    //     .await
    //     .expect("failed to start server");

    let router = Router::new();

    info!("Listening on {}", address);

    axum_server::bind(address)
        .serve(router.into_make_service())
        .await
}
