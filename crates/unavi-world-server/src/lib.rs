//! Server for running multiplayer instances of worlds over WebTransport.

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

mod cert;
mod connection;

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub port: u16,
}

pub async fn start(opts: ServerOptions) -> std::io::Result<()> {
    let opts = Arc::new(opts);

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), opts.port);

    let identity = cert::generate_tls_identity();
    let options = connection::WorldOptions {
        address,
        identity: &identity,
    };

    connection::start_server(options).await.unwrap();

    Ok(())
}
