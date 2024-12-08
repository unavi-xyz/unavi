//! Unified CLI tool for running UNAVI servers.
//!
//! ## Usage
//!
//! ```bash
//! unavi-server --help
//! ```

use clap::Parser;
use dwn::{stores::NativeDbStore, Dwn};
use tracing::{error, Level};
use unavi_server::{StartOptions, Storage, STORAGE_PATH};

#[tokio::main]
async fn main() {
    let args = unavi_server::Args::parse();

    let log_level = if args.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    let db = match &args.storage {
        Storage::Filesystem => NativeDbStore::new(STORAGE_PATH.clone()).unwrap(),
        Storage::Memory => NativeDbStore::new_in_memory().unwrap(),
    };
    let dwn = Dwn::from(db);

    if let Err(e) = unavi_server::start(args, StartOptions::default(), dwn).await {
        error!("{}", e);
    };
}
