use std::time::Duration;

use bevy::{
    prelude::*,
    time::common_conditions::on_timer,
};
use unavi_devtools::tabs::panel_active;

pub mod conn;
mod network;
mod state;

pub struct SpaceDevToolsPlugin;

impl Plugin for SpaceDevToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<network::NetSampler>()
            .init_resource::<state::StateSelection>()
            .init_resource::<state::SelectorPeers>()
            .add_systems(Startup, (network::spawn, state::spawn))
            .add_systems(
                Update,
                (
                    network::update
                        .run_if(panel_active::<network::NetworkPanel>)
                        .run_if(on_timer(Duration::from_secs(1))),
                    state::sync_selector.run_if(panel_active::<state::StatePanel>),
                    state::handle_select.run_if(panel_active::<state::StatePanel>),
                    state::render
                        .run_if(panel_active::<state::StatePanel>)
                        .run_if(on_timer(Duration::from_millis(500))),
                ),
            );
    }
}

/// Short hex prefix of a 32-byte id, for compact display.
fn short(bytes: &[u8]) -> String {
    format!("{:02x}{:02x}{:02x}", bytes[0], bytes[1], bytes[2])
}
