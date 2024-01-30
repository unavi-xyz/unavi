use std::net::SocketAddr;

use axum::Server;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let router = unavi_server_web::router().await;
    let address = SocketAddr::from(([127, 0, 0, 1], 3000));

    info!("Listening on {}", address);

    if let Err(e) = Server::bind(&address)
        .serve(router.into_make_service())
        .await
    {
        error!("Server error: {}", e);
    }
}
