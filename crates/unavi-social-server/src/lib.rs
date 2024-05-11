//! Server for running social protocols.
//! Hosts a DWN, provides login APIs, and more.

use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use dwn::{store::SurrealStore, DWN};
use surrealdb::{
    engine::local::{Mem, SurrealKV},
    Surreal,
};
use tracing::info;

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub domain: String,
    pub port: u16,
    pub storage: Storage,
}

#[derive(Debug, Clone)]
pub enum Storage {
    /// Path to a directory to store data within.
    Path(String),
    Memory,
}

pub async fn start(opts: ServerOptions) -> std::io::Result<()> {
    let opts = Arc::new(opts);

    let store = match &opts.storage {
        Storage::Path(path) => {
            let db_path = format!("{}/db", path);
            std::fs::create_dir_all(&db_path).unwrap();
            let db = Surreal::new::<SurrealKV>(db_path).await.unwrap();
            SurrealStore::new(db).await.unwrap()
        }
        Storage::Memory => {
            let db = Surreal::new::<Mem>(()).await.unwrap();
            SurrealStore::new(db).await.unwrap()
        }
    };

    let dwn = Arc::new(DWN::from(store));

    let addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), opts.port);
    let router = dwn_server::router(dwn.clone());

    info!("Listening on {}", addr);

    axum_server::bind(addr)
        .serve(router.into_make_service())
        .await
}
