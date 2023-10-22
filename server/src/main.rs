#[cfg(feature = "web")]
mod web;

#[tokio::main]
async fn main() {
    #[cfg(feature = "web")]
    let (router, addr) = web::router().await;

    #[cfg(not(feature = "web"))]
    let (router, addr) = {
        use axum::Router;
        use std::net::SocketAddr;

        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        let router = Router::new();

        (router, addr)
    };

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}
