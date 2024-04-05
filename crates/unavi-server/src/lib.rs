use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum_server::tls_rustls::RustlsConfig;
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
    pub port_did_host: u16,
    pub port_dwn: u16,
    pub port_world_host: u16,
    pub port_world_registry: u16,
}

pub async fn start(opts: ServerOptions) -> Result<(), Box<dyn std::error::Error>> {
    let opts = Arc::new(opts);

    std::fs::create_dir_all(DB_DIR).unwrap();

    let db = Surreal::new::<SpeeDb>(DB_DIR).await.unwrap();
    let dwn = Arc::new(DWN::from(SurrealStore::from(db)));

    let domains = vec!["localhost".to_string()];

    if opts.enable_did_host {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), opts.port_did_host);
        let domains = domains.clone();

        tokio::spawn(
            async move {
                let router = did_host::router();

                info!("Listening on {}", addr);

                let config = self_signed_config(domains).await;
                axum_server::bind_rustls(addr, config)
                    .serve(router.into_make_service())
                    .await
                    .unwrap();
            }
            .instrument(info_span!("did_host")),
        );
    }

    if opts.enable_dwn {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), opts.port_dwn);
        let domains = domains.clone();
        let dwn = dwn.clone();

        tokio::spawn(
            async move {
                let router = dwn_server::router(dwn);

                info!("Listening on {}", addr);

                let config = self_signed_config(domains).await;
                axum_server::bind_rustls(addr, config)
                    .serve(router.into_make_service())
                    .await
                    .unwrap();
            }
            .instrument(info_span!("dwn")),
        );
    }

    if opts.enable_world_host {
        let opts = opts.clone();

        tokio::spawn(
            async move {
                let addr = format!("0.0.0.0:{}", opts.port_world_host);
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
        let dwn = dwn.clone();
        let span = info_span!("world_registry");
        let span_clone = span.clone();

        let addr = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            opts.port_world_registry,
        );

        tokio::spawn(
            async move {
                let (router, create_registry) =
                    world_registry::router(dwn, &addr.clone().to_string()).await;

                tokio::spawn(
                    async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        create_registry.await;
                    }
                    .instrument(span_clone),
                );

                info!("Listening on {}", addr);

                let config = self_signed_config(domains).await;
                axum_server::bind_rustls(addr, config)
                    .serve(router.into_make_service())
                    .await
                    .unwrap();
            }
            .instrument(span),
        );
    }

    Ok(())
}

async fn self_signed_config(domains: Vec<String>) -> RustlsConfig {
    let cert = rcgen::generate_simple_self_signed(domains).unwrap();
    let cert_der = cert.serialize_der().unwrap();
    let key_der = cert.serialize_private_key_der();

    RustlsConfig::from_der(vec![cert_der], key_der)
        .await
        .unwrap()
}
