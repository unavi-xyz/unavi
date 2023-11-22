use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;

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
        match world::start_server(world::WorldOptions {
            address: opts.address,
            cert_pair: world::CertPair {
                cert: rustls::Certificate(world::cert::new_ca().serialize_der().unwrap()),
                key: rustls::PrivateKey(world::cert::new_ca().serialize_private_key_der()),
            },
        })
        .await
        {
            Ok(_) => println!("World server exited"),
            Err(e) => println!("Error: {}", e),
        }
    });

    println!("Listening on {}", opts.address);

    axum::Server::bind(&opts.address)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}

async fn ping() -> impl IntoResponse {
    "pong"
}
