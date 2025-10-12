use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use clap::Parser;
use tracing::error;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long, default_value_t = 8080)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    tracing_subscriber::fmt().init();

    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), args.port));

    if let Err(e) = unavi_server::run_server(addr).await {
        error!("{e:?}");
    }
}
