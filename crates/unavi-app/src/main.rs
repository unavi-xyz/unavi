#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::{Parser, ValueEnum};
use dwn::{stores::NativeDbStore, Dwn};
use tracing::Level;
use unavi_app::StartOptions;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Enables debug physics visuals.
    #[arg(long)]
    debug_physics: bool,

    /// Minimum log level.
    #[arg(long, default_value_t, value_enum)]
    log_level: LogLevel,

    /// Disables automatic updates.
    #[arg(long)]
    no_update: bool,

    #[arg(long, default_value = "filesystem")]
    storage: Storage,

    /// Enables XR mode.
    #[arg(long)]
    xr: bool,
}

impl Args {
    fn to_options(&self) -> StartOptions {
        let log_level = match self.log_level {
            LogLevel::Info => Level::INFO,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Trace => Level::TRACE,
        };

        StartOptions {
            debug_physics: self.debug_physics,
            log_level,
            xr: self.xr,
        }
    }
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    use unavi_app::native::update::check_for_updates;

    let args = Args::parse();

    if !args.no_update {
        if let Err(e) = check_for_updates() {
            panic!("Failed to update: {}", e);
        };
    }

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime");

    rt.block_on(async {
        let db = match args.storage {
            Storage::Filesystem => NativeDbStore::new_in_memory(),
            Storage::Memory => NativeDbStore::new_in_memory(),
        }
        .expect("Failed to create db");

        let dwn = Dwn::from(db);

        unavi_app::start(dwn, args.to_options()).await;
    });
}

#[derive(ValueEnum, Clone, Debug, Default)]
enum LogLevel {
    #[default]
    Info,
    Debug,
    Trace,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Storage {
    Filesystem,
    Memory,
}
