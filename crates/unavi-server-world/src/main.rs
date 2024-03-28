use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let address = SocketAddr::from(([127, 0, 0, 1], 3001));
    let identity = unavi_server_world::cert::generate_tls_identity();

    if let Err(e) = unavi_server_world::start_server(unavi_server_world::WorldOptions {
        address,
        identity: &identity,
    })
    .await
    {
        tracing::error!("Server error: {}", e);
    }
}
