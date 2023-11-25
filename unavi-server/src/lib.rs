use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;
use tracing::{error, info};

#[cfg(feature = "web")]
pub mod web;
#[cfg(feature = "world")]
pub mod world;

pub struct ServerOptions {
    pub address: SocketAddr,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            address: SocketAddr::from(([127, 0, 0, 1], 3000)),
        }
    }
}

pub async fn start_server(opts: ServerOptions) -> Result<(), Box<dyn std::error::Error>> {
    let router = Router::new().route("/ping", get(ping));

    #[cfg(feature = "web")]
    let router = router.merge(web::router().await);

    #[cfg(feature = "world")]
    tokio::spawn(async move {
        let ca = world::cert::new_ca();

        if let Err(e) = world::start_server(world::WorldOptions {
            address: opts.address,
            cert_pair: world::CertPair {
                cert: ca.serialize_der().unwrap(),
                key: ca.serialize_private_key_der(),
            },
        })
        .await
        {
            error!("World server: {}", e);
        }
    });

    info!("Listening on {}", opts.address);

    axum::Server::bind(&opts.address)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

async fn ping() -> impl IntoResponse {
    "pong"
}
