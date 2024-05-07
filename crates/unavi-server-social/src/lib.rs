//! Server for running social protocols.
//! Hosts a DWN, provides login APIs, and more.

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use dwn::{store::SurrealStore, DWN};
use surrealdb::{engine::local::SurrealKV, Surreal};
use tracing::info;

mod world_host;

const DB_DIR: &str = ".unavi/server/social/db";

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub domain: String,
    pub enable_world_host: bool,
    pub port: u16,
}

pub async fn start(opts: ServerOptions) -> std::io::Result<()> {
    let opts = Arc::new(opts);

    std::fs::create_dir_all(DB_DIR).unwrap();

    let db = Surreal::new::<SurrealKV>(DB_DIR).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = Arc::new(DWN::from(store));

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), opts.port);
    let mut router = dwn_server::router(dwn.clone());

    if opts.enable_world_host {
        let (world_host_router, create_world_host) =
            world_host::router(dwn, opts.domain.clone()).await;

        router = router.merge(world_host_router);

        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            create_world_host.await;
        });
    }

    info!("Listening on {}", addr);

    axum_server::bind(addr)
        .serve(router.into_make_service())
        .await
}
