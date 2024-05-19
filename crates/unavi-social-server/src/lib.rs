//! Server for running social protocols.
//! Hosts a DWN, provides login APIs, and more.

use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use dwn::{
    store::{DataStore, MessageStore},
    DWN,
};
use tracing::info;

#[derive(Clone)]
pub struct ServerOptions<D: DataStore, M: MessageStore> {
    pub domain: String,
    pub port: u16,
    pub dwn: Arc<DWN<D, M>>,
}

pub async fn start(
    opts: ServerOptions<impl DataStore + 'static, impl MessageStore + 'static>,
) -> std::io::Result<()> {
    let opts = Arc::new(opts);

    let addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), opts.port);
    let router = dwn_server::router(opts.dwn.clone());

    info!("Listening on {}", addr);

    axum_server::bind(addr)
        .serve(router.into_make_service())
        .await
}
