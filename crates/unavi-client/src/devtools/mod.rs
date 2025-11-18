//! Developer tools plugin.

use bevy::prelude::*;

pub mod events;
mod network_stats;
mod network_ui;

pub use network_stats::NetworkStats;

pub struct DevToolsPlugin {
    pub enabled: bool,
}

impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        if !self.enabled {
            return;
        }

        app.init_resource::<NetworkStats>()
            .add_systems(
                FixedUpdate,
                (
                    network_stats::collect_network_events,
                    network_stats::update_bandwidth_stats,
                    network_stats::update_tickrate_stats,
                    network_stats::detect_dropped_frames,
                ),
            )
            .add_systems(Startup, network_ui::spawn_devtools_overlay)
            .add_systems(Update, network_ui::update_network_stats_text);
    }
}
