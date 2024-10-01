//! Server for running social protocols.
//! Hosts a DWN, provides login APIs, and more.

use std::net::{Ipv4Addr, SocketAddr};

use dwn::DWN;
use tracing::info;

#[derive(Clone)]
pub struct ServerOptions {
    pub dwn: DWN,
    pub port: u16,
}

pub async fn start(opts: ServerOptions) -> std::io::Result<()> {
    let addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), opts.port);
    let router = dwn_server::router(opts.dwn.clone());

    info!("Listening on {}", addr);

    axum_server::bind(addr)
        .serve(router.into_make_service())
        .await
}
