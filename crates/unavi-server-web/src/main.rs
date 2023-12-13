use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let router = unavi_server_web::router().await;
    let address = SocketAddr::from(([127, 0, 0, 1], 3000));

    if let Err(e) = axum::Server::bind(&address)
        .serve(router.into_make_service())
        .await
    {
        tracing::error!("Server error: {}", e);
    }
}
