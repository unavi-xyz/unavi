//! Creates a world host DID and corresponding DWN protocol.
//! Hosts the DID document over HTTP, and routes incoming WebTransport requests to the correct
//! world server.

use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use axum::{routing::get, Json, Router};
use did::ActorOptions;
use dwn::{store::SurrealStore, DWN};
use surrealdb::{
    engine::local::{Mem, SurrealKV},
    Surreal,
};
use tracing::{error, info};

mod did;
mod world_host;

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub domain: String,
    pub dwn_url: String,
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

    let mut actor = did::create_actor(ActorOptions {
        domain: opts.domain.clone(),
        dwn,
        storage: opts.storage.clone(),
    });
    actor.add_remote(opts.dwn_url.clone());

    let document = did::document::create_document(&actor, opts.dwn_url.clone());

    let router = Router::new().route(
        "/.well-known/did.json",
        get(|| async move { Json(document.clone()) }),
    );

    let addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), opts.port);
    let server = tokio::spawn(axum_server::bind(addr).serve(router.into_make_service()));

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Server needs to be running, hosting the DID, before we can interact with DWN.
    world_host::create_world_host(&actor, &opts.dwn_url).await;

    let mut synced = false;

    while !synced {
        match actor.sync().await {
            Ok(()) => {
                info!("Sync successful.");
                synced = true;
            }
            Err(e) => {
                error!("Failed to sync: {}", e);
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    }

    server.await?
}
