//! Creates a world host DID and corresponding DWN protocol.
//! Hosts the DID document over HTTP, and routes incoming WebTransport requests to the correct
//! world server.

use std::{
    net::{Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{routing::get, Json, Router};
use dwn::{store::SurrealStore, DWN};
use surrealdb::{engine::local::SurrealKV, Surreal};

mod identity;
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

    let domain_name = &opts.domain.split(':').next().unwrap();
    let dwn_url = if *domain_name == "localhost" {
        format!("http://{}", opts.domain)
    } else {
        format!("https://{}", opts.domain)
    };

    let mut actor = identity::create_actor(opts.domain.clone(), dwn);
    actor.add_remote(dwn_url.clone());

    let document = identity::document::create_document(&actor, dwn_url.clone());

    let router = Router::new().route(
        "/.well-known/did.json",
        get(|| async move { Json(document.clone()) }),
    );

    world_host::create_world_host(&actor, &dwn_url).await;

    actor.sync().await.unwrap();

    let addr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), opts.port);

    axum_server::bind(addr)
        .serve(router.into_make_service())
        .await
}
