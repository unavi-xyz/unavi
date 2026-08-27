use std::time::Duration;

use bevy::{
    prelude::*,
    time::common_conditions::on_timer,
};
use unavi_devtools::tabs::panel_active;

pub mod conn;
mod inspect;
mod network;

pub struct SpaceDevToolsPlugin;

impl Plugin for SpaceDevToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<network::NetSampler>()
            .init_resource::<inspect::CurrentPage>()
            .init_resource::<inspect::History>()
            .init_resource::<inspect::Expanded>()
            .init_resource::<inspect::RenderedPage>()
            .init_resource::<inspect::sidebar::SidebarEntries>()
            .add_systems(Startup, (network::spawn, inspect::spawn))
            .add_observer(inspect::handle_link)
            .add_observer(inspect::handle_back)
            .add_observer(inspect::handle_expand)
            .add_observer(inspect::handle_rung)
            .add_systems(
                Update,
                (
                    network::update.run_if(panel_active::<network::NetworkPanel>),
                    inspect::sidebar::sync.run_if(
                        panel_active::<inspect::InspectPanel>
                            .and_then(on_timer(Duration::from_millis(500))),
                    ),
                    inspect::sidebar::highlight.run_if(panel_active::<inspect::InspectPanel>),
                    inspect::render.run_if(
                        panel_active::<inspect::InspectPanel>.and_then(
                            on_timer(Duration::from_millis(500))
                                .or_else(resource_changed::<inspect::CurrentPage>)
                                .or_else(resource_changed::<inspect::Expanded>)
                                .or_else(resource_changed::<inspect::History>),
                        ),
                    ),
                    inspect::widgets::refresh_times.run_if(
                        panel_active::<inspect::InspectPanel>
                            .and_then(on_timer(Duration::from_secs(1))),
                    ),
                ),
            );
    }
}

/// Short hex prefix of a 32-byte id, for compact display.
fn short(bytes: &[u8]) -> String {
    format!("{:02x}{:02x}{:02x}", bytes[0], bytes[1], bytes[2])
}
