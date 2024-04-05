use clap::Parser;
use tracing::{error, Level};
use unavi_server::ServerOptions;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Enables debug logging.
    #[arg(long)]
    debug: bool,

    /// Hosts DID documents over HTTP using `did:web`.
    /// Provides login APIs for users to create and manage their DIDs.
    #[arg(long)]
    enable_did_host: bool,

    /// Hosts an HTTP API for the Decentralized Web Node (DWN).
    #[arg(long)]
    enable_dwn: bool,

    /// Hosts multiplayer instances of worlds.
    #[arg(long)]
    enable_world_host: bool,

    /// Creates a world registry within the DWN.
    #[arg(long)]
    enable_world_registry: bool,

    #[arg(short, long, default_value = "443")]
    port: u16,

    #[arg(long, default_value = "8082")]
    port_world_host: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut log = tracing_subscriber::fmt();
    if args.debug {
        log = log.with_max_level(Level::DEBUG);
    }
    log.init();

    if args.enable_world_registry && !args.enable_dwn {
        error!("--enable-world-registry requires --enable-dwn");
        return;
    }

    let options = ServerOptions {
        enable_did_host: args.enable_did_host,
        enable_dwn: args.enable_dwn,
        enable_world_host: args.enable_world_host,
        enable_world_registry: args.enable_world_registry,
        port: args.port,
        port_world_host: args.port_world_host,
    };

    unavi_server::start(options)
        .await
        .expect("Failed to start server");

    tokio::signal::ctrl_c().await.unwrap();
}
