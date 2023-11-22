#[tokio::main]
async fn main() {
    println!("Features:");
    let feat_web = cfg!(feature = "web");
    println!("- web: {}", feat_web);
    let feat_world = cfg!(feature = "world");
    println!("- world: {}", feat_world);

    match unavi_server::start_server(unavi_server::ServerOptions::default()).await {
        Ok(_) => println!("Server exited successfully"),
        Err(e) => println!("Server exited with error: {}", e),
    }
}
