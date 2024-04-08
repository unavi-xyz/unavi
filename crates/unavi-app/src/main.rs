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

    let mut args = Args { debug: false };

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

#[cfg(not(target_family = "wasm"))]
#[tokio::main]
async fn main() {
    tokio::task::spawn_blocking(|| {
        if let Err(e) = update() {
            println!("Error while updating: {}", e);
        }
    })
    .await
    .unwrap();

    const DB_PATH: &str = ".unavi/app-db";

    std::fs::create_dir_all(DB_PATH).expect("Failed to create database dir.");

    let db = Surreal::new::<surrealdb::engine::local::SpeeDb>(DB_PATH)
        .await
        .expect("Failed to create SurrealDB.");

    let args = Args::parse();
    let opts = args_to_options(args);

    unavi_app::start(db, opts).await
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Enables debug logging and rendering.
    #[arg(long)]
    debug: bool,
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
        ..Default::default()
    }
}

#[cfg(not(target_family = "wasm"))]
fn update() -> Result<(), Box<dyn std::error::Error>> {
    let status = self_update::backends::github::Update::configure()
        .repo_owner("unavi-xyz")
        .repo_name("unavi")
        .bin_name("unavi-app")
        .show_download_progress(true)
        .current_version(env!("CARGO_PKG_VERSION"))
        .build()?
        .update()?;

    if status.updated() {
        println!("Updated to {}. Restarting.", status.version());

        if let Err(e) = restart_app() {
            eprintln!("Failed to restart application: {}", e);
        }

        std::process::exit(0);
    }

    Ok(())
}

#[cfg(not(target_family = "wasm"))]
fn restart_app() -> std::io::Result<()> {
    let exe = std::env::current_exe()?;
    std::process::Command::new(exe).spawn()?;
    Ok(())
}
