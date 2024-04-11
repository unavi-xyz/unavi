use clap::Parser;
use tracing::Level;
use unavi_server::ServerOptions;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Enables debug logging.
    #[arg(long)]
    debug: bool,

    /// The domain of the server.
    /// Used to generate DIDs for the server.
    #[arg(long, default_value = "localhost:<port>")]
    domain: String,

    /// Provides login APIs for users to create and manage their DIDs.
    /// Hosts user DID documents over HTTP.
    #[arg(long)]
    enable_did_host: bool,

    /// Provides an HTTP API for the Decentralized Web Node (DWN).
    #[arg(long)]
    enable_dwn: bool,

    /// Hosts multiplayer instances of worlds within the connected registry.
    #[arg(long)]
    enable_world_host: bool,

    /// Creates a world registry within the DWN.
    /// Hosts the registry DID document over HTTP.
    #[arg(long)]
    enable_world_registry: bool,

    #[arg(short, long, default_value = "3000")]
    port: u16,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut log = tracing_subscriber::fmt();
    if args.debug {
        log = log.with_max_level(Level::DEBUG);
    }
    log.init();

    let domain = if args.domain == "localhost:<port>" {
        format!("localhost:{}", args.port)
    } else {
        args.domain
    };

    let options = ServerOptions {
        domain,
        enable_did_host: args.enable_did_host,
        enable_dwn: args.enable_dwn,
        enable_world_host: args.enable_world_host,
        enable_world_registry: args.enable_world_registry,
        port: args.port,
    };

    unavi_server::start(options)
        .await
        .expect("Failed to start server");

    tokio::signal::ctrl_c().await.unwrap();
}
