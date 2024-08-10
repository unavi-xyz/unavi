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
    engine::local::{Mem, SurrealKV},
    Surreal,
};
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

    let store = match &args.storage {
        Storage::Filesystem => {
            let db = Surreal::new::<SurrealKV>(STORAGE_PATH.clone())
                .await
                .unwrap();
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
