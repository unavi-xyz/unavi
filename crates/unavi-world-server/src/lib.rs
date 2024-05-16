//! Server for running multiplayer instances of worlds over WebTransport.

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use tracing::{info, info_span, Instrument};
use wtransport::{Endpoint, Identity, ServerConfig};

mod connection;
mod rpc;

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub port: u16,
    pub domain: String,
}

pub async fn start(opts: ServerOptions) -> std::io::Result<()> {
    let opts = Arc::new(opts);

    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), opts.port);

    let config = ServerConfig::builder()
        .with_bind_address(address)
        .with_identity(&Identity::self_signed(["localhost", "127.0.0.1", &opts.domain]).unwrap())
        .build();

    let endpoint = Endpoint::server(config)?;
    let endpoint = xwt_wtransport::Endpoint(endpoint);
    info!("Listening on {}", address);

    for id in 0.. {
        let incoming_session = xwt_wtransport::IncomingSession(endpoint.accept().await);

        tokio::spawn(
            connection::handle_connection(incoming_session)
                .instrument(info_span!("Connection", id)),
        );
    }

    info!("Finished.");
    Ok(())
}
