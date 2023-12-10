use axum::Router;
use tracing::info;

use super::ServerOptions;

pub async fn start_server(opts: &ServerOptions) -> Result<(), Box<dyn std::error::Error>> {
    tracing::span!(tracing::Level::INFO, "axum_server");

    let router = Router::new();

    #[cfg(feature = "web")]
    let router = router.merge(unavi_server_web::router().await);

    info!("Listening on {}", opts.address);

    axum::Server::bind(&opts.address)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
