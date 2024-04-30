#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;
use surrealdb::Surreal;
use tracing::Level;
use unavi_app::StartOptions;

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub async fn wasm_start() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let location = document.location().unwrap();
    let search = location.search().unwrap();
    let params = web_sys::UrlSearchParams::new_with_str(&search).unwrap();

    let mut args = Args {
        debug: false,
        force_update: false,
    };

    if let Some(value) = params.get("debug") {
        if let Ok(value) = value.parse() {
            args.debug = value;
        }
    }

    let db = Surreal::new::<surrealdb::engine::local::IndxDb>("unavi")
        .await
        .expect("Failed to create SurrealDB.");

    let opts = args_to_options(args);

    unavi_app::start(db, opts).await
}

#[cfg(target_family = "wasm")]
fn main() {}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Enables debug logging and rendering.
    #[arg(long)]
    debug: bool,
    /// Forces an update from the latest release, even if the versions match.
    #[arg(long)]
    force_update: bool,
}

const DB_PATH: &str = ".unavi/app-db";

#[cfg(not(target_family = "wasm"))]
#[tokio::main]
async fn main() {
    let args = Args::parse();

    #[cfg(feature = "self_update")]
    {
        let force_update = args.force_update;
        tokio::task::spawn_blocking(move || {
            if let Err(e) = unavi_app::update::check_for_updates(force_update) {
                println!("Error while updating: {}", e);
            }
        })
        .await
        .unwrap();
    }

    std::fs::create_dir_all(DB_PATH).expect("Failed to create database dir.");

    let db = Surreal::new::<surrealdb::engine::local::SurrealKV>(DB_PATH)
        .await
        .expect("Failed to create SurrealDB.");

    let opts = args_to_options(args);
    unavi_app::start(db, opts).await
}

fn args_to_options(args: Args) -> StartOptions {
    let log_level = if args.debug {
        Level::DEBUG
    } else {
        Level::INFO
    };

    StartOptions {
        debug_physics: args.debug,
        log_level,
    }
}
