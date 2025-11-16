#![windows_subsystem = "windows"]

use bevy::prelude::*;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// Run the local DWN in-memory instead of writing to disk.
    #[arg(long, default_value_t = false)]
    in_memory: bool,
}

fn main() {
    let args = Args::parse();

    App::new()
        .add_plugins(unavi_client::UnaviPlugin {
            in_memory: args.in_memory,
        })
        .run();
}
