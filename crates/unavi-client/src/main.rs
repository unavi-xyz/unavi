// #![windows_subsystem = "windows"]

use std::time::Duration;

use bevy::prelude::*;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Run the local DWN in-memory instead of writing to disk.
    #[arg(long, default_value_t = false)]
    in_memory: bool,

    /// Enable FPS counter.
    #[cfg(feature = "devtools-bevy")]
    #[arg(long, default_value_t = false)]
    debug_fps: bool,

    /// Enable debug network monitoring (shows bandwidth, tickrate, etc.).
    #[cfg(feature = "devtools-network")]
    #[arg(long, default_value_t = false)]
    debug_network: bool,

    /// Enable physics debug gizmos.
    #[cfg(feature = "devtools-bevy")]
    #[arg(long, default_value_t = false)]
    debug_physics: bool,
}

fn main() {
    let args = Args::parse();

    App::new()
        .add_plugins(unavi_client::UnaviPlugin {
            in_memory: args.in_memory,
            #[cfg(feature = "devtools-bevy")]
            debug_fps: args.debug_fps,
            #[cfg(feature = "devtools-network")]
            debug_network: args.debug_network,
            #[cfg(feature = "devtools-bevy")]
            debug_physics: args.debug_physics,
        })
        .run();

    // Give time for other threads to finish.
    std::thread::sleep(Duration::from_millis(500));

    info!("Graceful exit");
}
