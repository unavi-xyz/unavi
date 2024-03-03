use std::net::SocketAddr;

use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let router = unavi_server_auth::router().await;
    let address = SocketAddr::from(([127, 0, 0, 1], 3002));

    info!("Listening on {}", address);

    let listener = match tokio::net::TcpListener::bind(&address).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind to address: {}", e);
            return;
        }
    };

    if let Err(e) = axum::serve(listener, router).await {
        error!("Server error: {}", e);
    }
}
