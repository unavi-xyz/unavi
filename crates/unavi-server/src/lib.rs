use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::Router;
use dwn::{store::SurrealStore, DWN};
use surrealdb::{engine::local::SpeeDb, Surreal};
use tracing::{info, info_span, Instrument};

mod did_host;
mod world_host;
mod world_registry;

const DB_DIR: &str = ".unavi/db";

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub enable_did_host: bool,
    pub enable_dwn: bool,
    pub enable_world_host: bool,
    pub enable_world_registry: bool,
    pub port: u16,
}

pub async fn start(opts: ServerOptions) -> std::io::Result<()> {
    let opts = Arc::new(opts);

    std::fs::create_dir_all(DB_DIR).unwrap();

    let db = Surreal::new::<SpeeDb>(DB_DIR).await.unwrap();
    let dwn = Arc::new(DWN::from(SurrealStore::from(db)));

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), opts.port);
    let mut router = Router::new();

    if opts.enable_did_host {
        router = router.merge(did_host::router());
    }

    if opts.enable_dwn {
        router = router.merge(dwn_server::router(dwn.clone()));
    }

    if opts.enable_world_host {
        let opts = opts.clone();

        tokio::spawn(
            async move {
                let addr = format!("0.0.0.0:{}", 3001);
                let identity = world_host::cert::generate_tls_identity();
                let options = world_host::WorldOptions {
                    address: addr.parse().unwrap(),
                    identity: &identity,
                };

                info!("Listening on {}", addr);

                world_host::start_server(options)
                    .await
                    .expect("failed to start server");
            }
            .instrument(info_span!("world_host")),
        );
    }

    if opts.enable_world_registry {
        let (registry_router, create_registry) =
            world_registry::router(dwn, &addr.clone().to_string()).await;

        router = router.merge(registry_router);

        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            create_registry.await;
        });
    }

    info!("Listening on {}", addr);

    axum_server::bind(addr)
        .serve(router.into_make_service())
        .await
}
