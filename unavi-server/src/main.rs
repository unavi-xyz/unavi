#[cfg(feature = "web")]
mod axum_server;

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

    #[cfg(feature = "web")]
    axum_server::axum_server().await;

    #[cfg(feature = "world")]
    world::world_server().await;
}
