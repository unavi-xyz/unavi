use axum::Router;
use tracing::info;

use super::ServerOptions;

pub async fn start(opts: &ServerOptions) -> Result<(), Box<dyn std::error::Error>> {
    tracing::span!(tracing::Level::INFO, "axum_server");

    let router = Router::new();

    #[cfg(feature = "auth")]
    let router = router.merge(unavi_server_auth::router().await);

    #[cfg(feature = "web")]
    let router = router.merge(unavi_server_web::router().await);

    info!("Listening on {}", opts.axum_addr);

    axum::Server::bind(&opts.axum_addr)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
