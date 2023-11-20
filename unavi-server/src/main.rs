#[tokio::main]
async fn main() {
    println!("Features:");
    let feat_web = cfg!(feature = "web");
    println!("- web: {}", feat_web);
    let feat_world = cfg!(feature = "world");
    println!("- world: {}", feat_world);

    unavi_server::server().await;
}
