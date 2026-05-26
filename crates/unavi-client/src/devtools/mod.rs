use bevy::prelude::*;

#[cfg(feature = "devtools-network")]
pub mod events;
#[cfg(feature = "devtools-network")]
mod network_stats;
#[cfg(feature = "devtools-network")]
mod network_ui;

pub struct DevToolsPlugin {
    #[allow(dead_code)]
    pub inspector: bool,
    #[allow(dead_code)]
    pub network: bool,
}

impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "devtools-inspector")]
        if self.inspector {
            app.add_plugins((
                bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
                bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
            ));
        }

        #[cfg(feature = "devtools-network")]
        if self.network {
            app.init_resource::<network_stats::NetworkStats>()
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
}
