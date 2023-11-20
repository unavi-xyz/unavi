use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;

pub async fn axum_server() {
    let router = Router::new().route("/ping", get(ping));

    #[cfg(feature = "web")]
    let router = router.merge(super::web::router().await);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

async fn ping() -> impl IntoResponse {
    "pong"
}
