//! Unified CLI tool for running UNAVI servers.
//!
//! ## Usage
//!
//! ```bash
//! unavi-server --help
//! ```

use std::sync::Arc;

use clap::Parser;
use dwn::{store::SurrealStore, DWN};
use surrealdb::{
    engine::local::{Mem, SurrealKv},
    Surreal,
};
use tracing::{error, Level};
use unavi_server::{process_args::process_args, StartOptions, Storage};

#[tokio::main]
async fn main() {
    let mut args = unavi_server::Args::parse();
    process_args(&mut args);

    let log_level = if args.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    let store = match &args.storage {
        Storage::Filesystem => {
            let db_path = format!("{}/db", args.path);
            std::fs::create_dir_all(&db_path).unwrap();
            let db = Surreal::new::<SurrealKv>(db_path).await.unwrap();
            SurrealStore::new(db).await.unwrap()
        }
        Storage::Memory => {
            let db = Surreal::new::<Mem>(()).await.unwrap();
            SurrealStore::new(db).await.unwrap()
        }
    };
    let dwn = Arc::new(DWN::from(store));

    if let Err(e) = unavi_server::start(args, StartOptions::default(), dwn).await {
        error!("{}", e);
    };
}
