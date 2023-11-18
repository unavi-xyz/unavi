use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;

#[cfg(feature = "web")]
mod web;
#[cfg(feature = "world")]
mod world;

#[tokio::main]
async fn main() {
    println!("Features:");
    let feat_web = cfg!(feature = "web");
    println!("- web: {}", feat_web);
    let feat_world = cfg!(feature = "world");
    println!("- world: {}", feat_world);

    let router = Router::new().route("/ping", get(ping));

    #[cfg(feature = "web")]
    let router = router.merge(web::router().await);

    #[cfg(feature = "world")]
    let router = router.merge(world::router().await);

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
