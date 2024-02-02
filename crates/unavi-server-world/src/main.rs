use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let address = SocketAddr::from(([127, 0, 0, 1], 3001));
    let certificate = unavi_server_world::cert::generate_certificate();

    if let Err(e) = unavi_server_world::start_server(unavi_server_world::WorldOptions {
        address,
        certificate,
    })
    .await
    {
        tracing::error!("Server error: {}", e);
    }
}
