use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;

#[cfg(feature = "web")]
mod web;

#[tokio::main]
async fn main() {
    println!("Features:");
    let feat_web = cfg!(feature = "web");
    println!("- web: {}", feat_web);

    let router = Router::new().route("/ping", get(ping));

    #[cfg(feature = "web")]
    let router = router.merge(web::router().await);

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
