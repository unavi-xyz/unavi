use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    info!("Features:");
    let feat_social = cfg!(feature = "social");
    info!("- social: {}", feat_social);
    let feat_web = cfg!(feature = "web");
    info!("- web: {}", feat_web);
    let feat_world = cfg!(feature = "world");
    info!("- world: {}", feat_world);

    if let Err(e) = unavi_server::start(unavi_server::ServerOptions::default()).await {
        error!(e);
    }

    tokio::signal::ctrl_c().await.unwrap();
}
