// #![windows_subsystem = "windows"]

use std::time::Duration;

use bevy::prelude::*;
use clap::Parser;
use tracing::Level;
use unavi_client::DebugFlags;

#[derive(Parser, Debug)]
#[command(version)]
#[allow(clippy::struct_excessive_bools)]
struct Args {
    /// Enable debug logging.
    #[arg(long, default_value_t = false)]
    debug_log: bool,

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

    /// Runs certain functions, like the local WDS, in-memory.
    /// Useful for running multiple clients on the same machine.
    #[arg(long, default_value_t = false)]
    in_memory: bool,

    /// Runs in XR mode.
    #[arg(long, default_value_t = false)]
    xr: bool,
}

fn main() {
    #[cfg(not(target_family = "wasm"))]
    let args = Args::parse();

    #[cfg(target_family = "wasm")]
    let args = {
        web_sys::console::log_1(&"parsing url params".into());

        let window = web_sys::window().expect("get window");
        let search = window.location().search().expect("get search");
        let params = web_sys::UrlSearchParams::new_with_str(&search).expect("parse search params");

        let mut argv = vec!["app".to_string()];

        for key in params.keys() {
            let Ok(key) = key else {
                continue;
            };
            let Some(key) = key.as_string() else {
                continue;
            };
            let Some(value) = params.get(&key) else {
                continue;
            };
            argv.push(format!("--{}", key));

            if !value.is_empty() {
                argv.push(value);
            }
        }

        web_sys::console::log_1(&format!("{argv:?}").into());

        match Args::try_parse_from(argv) {
            Ok(a) => a,
            Err(err) => {
                web_sys::console::warn_1(&format!("Error parsing params: {err}").into());
                Args::parse()
            }
        }
    };

    let log_level = if args.debug_log {
        Level::DEBUG
    } else {
        Level::INFO
    };

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
            debug,
            in_memory: args.in_memory,
            log_level,
            xr: args.xr,
        })
        .run();

    // Give time for other threads to finish.
    unavi_wasm_compat::sleep_thread(Duration::from_millis(200));

    info!("Graceful exit");
}
