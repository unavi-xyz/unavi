//! Server for running social protocols.
//! Hosts a DWN, provides login APIs, and more.

use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use dwn::{store::SurrealStore, DWN};
use surrealdb::{engine::local::SurrealKV, Surreal};
use tracing::info;

const ROOT_DIR: &str = ".unavi/server/social";

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub domain: String,
    pub port: u16,
}

pub async fn start(opts: ServerOptions) -> std::io::Result<()> {
    let opts = Arc::new(opts);

    let db_dir = format!("{}/db", ROOT_DIR);

    std::fs::create_dir_all(&db_dir).unwrap();

    let db = Surreal::new::<SurrealKV>(&db_dir).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = Arc::new(DWN::from(store));

    let addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), opts.port);
    let router = dwn_server::router(dwn.clone());

    info!("Listening on {}", addr);

    axum_server::bind(addr)
        .serve(router.into_make_service())
        .await
}
