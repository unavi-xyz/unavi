//! Unified CLI tool for running UNAVI servers.
//!
//! ## Usage
//!
//! ```bash
//! unavi-server --help
//! ```

use clap::Parser;
use tracing::{error, Level};

#[tokio::main]
async fn main() {
    let args = unavi_server::Args::parse();

    let log_level = if args.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    if let Err(e) = unavi_server::start(args).await {
        error!("{}", e);
    };
}
