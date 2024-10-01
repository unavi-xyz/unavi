//! Creates a world host DID and corresponding DWN protocol.
//! Hosts the DID document over HTTP, and routes incoming WebTransport requests to the correct
//! world server.

use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
    time::Duration,
};

use axum::{routing::get, Json, Router};
use did::ActorOptions;
use dwn::{actor::Actor, DWN};

use tracing::{error, info};

mod did;
mod world_host;

#[derive(Clone)]
pub struct ServerOptions {
    pub domain: String,
    pub dwn: DWN,
    pub port: u16,
    pub remote_dwn: String,
    pub remote_sync: bool,
    pub storage: Storage,
}

#[derive(Debug, Clone)]
pub enum Storage {
    /// Path to a directory to store data within.
    Path(PathBuf),
    Memory,
}

pub async fn start(opts: ServerOptions) -> std::io::Result<()> {
    let mut actor = did::create_actor(ActorOptions {
        domain: opts.domain.clone(),
        dwn: opts.dwn,
        storage: opts.storage.clone(),
    });

    if opts.remote_sync {
        actor.add_remote(opts.remote_dwn.clone());
    }

    let document = did::document::create_document(&actor, opts.remote_dwn.clone());

    let router = Router::new().route(
        "/.well-known/did.json",
        get(|| async move { Json(document.clone()) }),
    );

    let addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), opts.port);
    info!("Starting world host on {}", addr);
    let server = tokio::spawn(axum_server::bind(addr).serve(router.into_make_service()));

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Sync first.
    if opts.remote_sync {
        sync_retry(&mut actor).await;
    }

    // Interact with DWN.
    const LOCALHOST: &str = "localhost:";
    let connect_domain = if opts.domain.starts_with(LOCALHOST) {
        opts.domain.replace(LOCALHOST, "127.0.0.1:")
    } else {
        opts.domain
    };
    let connect_url = format!("https://{}", connect_domain);

    world_host::create_world_host(&actor, &connect_url).await;

    // Sync after.
    if opts.remote_sync {
        sync_retry(&mut actor).await;
    }

    server.await??;

    info!("Finished.");
    Ok(())
}

async fn sync_retry(actor: &mut Actor) {
    let mut synced = false;
    let mut delay = 3.0;

    while !synced {
        match actor.sync().await {
            Ok(()) => {
                info!("Sync successful.");
                synced = true;
            }
            Err(e) => {
                error!("Failed to sync: {}", e);
                tokio::time::sleep(Duration::from_secs_f32(delay)).await;
                delay *= 1.5;
            }
        }
    }
}
