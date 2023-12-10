use std::net::SocketAddr;

use wtransport::{tls::Certificate, Endpoint, ServerConfig};

pub mod cert;

pub struct WorldOptions {
    pub address: SocketAddr,
    pub certificate: Certificate,
}

pub async fn start_server(opts: WorldOptions) -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::builder()
        .with_bind_address(opts.address)
        .with_certificate(opts.certificate)
        .build();

    let endpoint = Endpoint::server(config)?;

    Ok(())
}
