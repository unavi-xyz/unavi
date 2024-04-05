use clap::Parser;
use tracing::error;
use unavi_server::ServerOptions;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
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

    #[arg(long, default_value = "8080")]
    port_did_host: Option<u16>,

    #[arg(long, default_value = "8081")]
    port_dwn: Option<u16>,

    #[arg(long, default_value = "8082")]
    port_world_host: Option<u16>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let args = Args::parse();

    if args.enable_world_registry && !args.enable_dwn {
        error!("--enable-world-registry requires --enable-dwn");
        return;
    }

    let options = ServerOptions {
        enable_did_host: args.enable_did_host,
        enable_dwn: args.enable_dwn,
        enable_world_registry: args.enable_world_registry,
        enable_world_host: args.enable_world_host,
        port_did_host: args.port_did_host.unwrap(),
        port_world_host: args.port_world_host.unwrap(),
        port_dwn: args.port_dwn.unwrap(),
    };

    if let Err(e) = unavi_server::start(options).await {
        error!(e);
    }

    tokio::signal::ctrl_c().await.unwrap();
}
