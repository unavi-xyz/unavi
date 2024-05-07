//! Unified CLI tool for running UNAVI servers.
//!
//! ## Usage
//!
//! ```bash
//! unavi-server --help
//! ```

use clap::{Parser, Subcommand};
use tracing::{error, Level};

#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// Enables debug logging.
    #[arg(long)]
    debug: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Social protocol.
    /// Hosts a DWN, login APIs, and more.
    Social {
        #[arg(long, default_value = "localhost:<port>")]
        domain: String,

        /// Creates a world host and protocol within the DWN.
        #[arg(long)]
        enable_world_host: bool,

        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
    /// World protocol.
    /// Hosts multiplayer instances of worlds.
    World {
        #[arg(short, long, default_value = "3001")]
        port: u16,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let log_level = if args.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    match args.command {
        Command::Social {
            domain,
            enable_world_host,
            port,
        } => {
            let domain = if domain == "localhost:<port>" {
                format!("localhost:{}", port)
            } else {
                domain
            };

            if let Err(e) = unavi_server_social::start(unavi_server_social::ServerOptions {
                domain,
                enable_world_host,
                port,
            })
            .await
            {
                error!("{}", e);
            };
        }
        Command::World { port } => {
            if let Err(e) =
                unavi_server_world::start(unavi_server_world::ServerOptions { port }).await
            {
                error!("{}", e);
            };
        }
    }
}
