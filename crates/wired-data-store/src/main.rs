use std::sync::LazyLock;

use clap::Parser;
use directories::ProjectDirs;
use iroh_tickets::endpoint::EndpointTicket;
use tracing::{Level, info};
use wired_data_store::DataStoreBuilder;

#[derive(Parser, Debug)]
#[command(version)]
#[allow(clippy::struct_excessive_bools)]
struct Args {
    /// Enable debug logging.
    #[arg(long, default_value_t = false)]
    debug: bool,

    /// Ephemeral mode doesn't store the endpoint key to disk.
    #[arg(long, default_value_t = false)]
    ephemeral: bool,

    /// Enable the `iroh-gossip` protocol, allowing the endpoint to be used
    /// for peer bootstrapping.
    #[arg(long, default_value_t = false)]
    gossip: bool,
}

static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "wired-data-store").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    dirs
});

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let level = if args.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };

    tracing_subscriber::fmt().with_max_level(level).init();

    tracing::info!("Initializing WDS");

    let store = DataStoreBuilder {
        ephemeral: args.ephemeral,
        data_dir: DIRS.data_local_dir().to_path_buf(),
        with_router: Some(Box::new(move |endpoint, r| {
            if args.gossip {
                let gossip = iroh_gossip::Gossip::builder().spawn(endpoint.clone());
                r.accept(iroh_gossip::ALPN, gossip)
            } else {
                r
            }
        })),
    }
    .build()
    .await?;

    let ticket = EndpointTicket::new(store.endpoint().addr());
    info!("Endpoint ticket: {ticket}");

    info!("Endpoint listening at:");
    for addr in store.endpoint().addr().ip_addrs() {
        info!("- {addr}");
    }

    tokio::signal::ctrl_c().await?;

    store.router().shutdown().await?;

    Ok(())
}
