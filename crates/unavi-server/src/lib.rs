use std::sync::Arc;

use tokio::net::TcpListener;
use tracing::{info, info_span, Instrument};

mod did_host;
mod world_host;

#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub enable_did_host: bool,
    pub enable_world_registry: bool,
    pub enable_world_host: bool,
    pub port_did_host: u16,
    pub port_world_host: u16,
    pub port_dwn: u16,
}

pub async fn start(opts: ServerOptions) -> Result<(), Box<dyn std::error::Error>> {
    let opts = Arc::new(opts);

    if opts.enable_did_host {
        let opts = opts.clone();

        tokio::spawn(
            async move {
                let router = did_host::router();

                let addr = format!("0.0.0.0:{}", opts.port_did_host);
                let listener = TcpListener::bind(addr.clone())
                    .await
                    .expect("failed to bind");

                info!("Listening on {}", addr);

                axum::serve(listener, router)
                    .await
                    .expect("failed to start server");
            }
            .instrument(info_span!("did_host")),
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

    Ok(())
}
