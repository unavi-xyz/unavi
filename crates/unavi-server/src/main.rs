use clap::Parser;
use tracing::{Level, error};
use unavi_server::ServerOptions;

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
        port: args.port,
        in_memory: false,
    })
    .await
    {
        error!("{e:?}");
    }
}
