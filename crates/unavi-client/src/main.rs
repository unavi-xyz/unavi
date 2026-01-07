// #![windows_subsystem = "windows"]

use std::{str::FromStr, time::Duration};

use bevy::prelude::*;
use blake3::Hash;
use clap::Parser;
use unavi_client::DebugFlags;

#[derive(Parser, Debug)]
#[command(version)]
#[allow(clippy::struct_excessive_bools)]
struct Args {
    /// ID of a space to join.
    #[arg(long)]
    join: Option<String>,

    /// Runs certain functions, like the local WDS, in-memory.
    /// Useful for running multiple clients on the same machine.
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

    let join = match args.join.as_ref().map(|t| Hash::from_str(t)) {
        Some(Ok(t)) => Some(t),
        Some(Err(e)) => {
            println!("Invalid space id: {e:?}");
            return;
        }
        None => None,
    };

    App::new()
        .add_plugins(unavi_client::UnaviPlugin {
            debug,
            in_memory: args.in_memory,
            initial_space: join,
        })
        .run();

    // Give time for other threads to finish.
    std::thread::sleep(Duration::from_millis(200));

    info!("Graceful exit");
}
