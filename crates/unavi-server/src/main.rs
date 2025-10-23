use clap::Parser;
use tracing::{Level, error};
use unavi_server::ServerOptions;
use xdid::methods::web::reqwest::Url;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long, default_value_t = 5000)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    if let Err(e) = unavi_server::run_server(ServerOptions {
        in_memory: false,
        port: args.port,
        remote_dwn: Url::parse(unavi_constants::REMOTE_DWN_URL).unwrap(),
    })
    .await
    {
        error!("{e:?}");
    }
}
