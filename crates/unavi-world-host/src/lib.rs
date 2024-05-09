//! Creates a world host DID and corresponding DWN protocol.
//! Hosts the DID document over HTTP, and routes incoming WebTransport requests to the correct
//! world server.

use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Duration,
};

use axum::{routing::get, Json, Router};
use dwn::{store::SurrealStore, DWN};
use surrealdb::{engine::local::SurrealKV, Surreal};
use tokio::time::sleep;
use tracing::{error, info};

mod did;
mod world_host;

const ROOT_DIR: &str = ".unavi/server/world-host";

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub domain: String,
    pub dwn_url: String,
    pub port: u16,
}

pub async fn start(opts: ServerOptions) -> std::io::Result<()> {
    let db_dir = format!("{}/db", ROOT_DIR);

    std::fs::create_dir_all(&db_dir).unwrap();

    let db = Surreal::new::<SurrealKV>(&db_dir).await.unwrap();
    let store = SurrealStore::new(db).await.unwrap();
    let dwn = Arc::new(DWN::from(store));

    let mut actor = did::create_actor(opts.domain.clone(), dwn);

    let document = did::document::create_document(&actor, opts.dwn_url.clone());

    let router = Router::new().route(
        "/.well-known/did.json",
        get(|| async move { Json(document.clone()) }),
    );

    let addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), opts.port);
    let server = tokio::spawn(axum_server::bind(addr).serve(router.into_make_service()));

    // Server needs to be running, hosting the DID, before we can interact with DWN.
    world_host::create_world_host(&actor, &opts.dwn_url).await;

    actor.add_remote(opts.dwn_url);

    let mut synced = false;

    while !synced {
        match actor.sync().await {
            Ok(()) => {
                info!("Sync successful.");
                synced = true;
            }
            Err(e) => {
                error!("Failed to sync: {}", e);
                sleep(Duration::from_secs(3)).await;
            }
        }
    }

    server.await?
}
