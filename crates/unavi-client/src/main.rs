// #![windows_subsystem = "windows"]

use std::time::Duration;

use bevy::prelude::*;
use clap::Parser;
use unavi_client::{DebugFlags, Storage};

#[derive(Parser, Debug)]
#[command(version)]
#[allow(clippy::struct_excessive_bools)]
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

    #[allow(unused_mut)]
    let mut debug = DebugFlags::empty();

    #[cfg(feature = "devtools-bevy")]
    {
        if args.debug_fps {
            debug |= DebugFlags::FPS;
        }
        if args.debug_physics {
            debug |= DebugFlags::PHYSICS;
        }
    }
    #[cfg(feature = "devtools-network")]
    {
        if args.debug_network {
            debug |= DebugFlags::NETWORK;
        }
    }

    App::new()
        .add_plugins(unavi_client::UnaviPlugin {
            storage: if args.in_memory {
                Storage::InMemory
            } else {
                Storage::Disk
            },
            debug,
        })
        .run();

    // Give time for other threads to finish.
    std::thread::sleep(Duration::from_millis(500));

    info!("Graceful exit");
}
