use bevy::prelude::*;

use crate::Flags;

mod event_gizmos;
// mod events;
// mod network_stats;
// mod network_ui;

pub struct DevToolsPlugin {
    pub flags: Flags,
}

impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        if self.flags.contains(Flags::DEBUG_EVENT) {
            event_gizmos::install_emit_observer();
            app.init_resource::<event_gizmos::EventPings>().add_systems(
                Update,
                (
                    event_gizmos::update_event_pings,
                    event_gizmos::draw_receptors,
                    event_gizmos::draw_event_pings,
                ),
            );
        }

        if self.flags.contains(Flags::DEBUG_FPS) {
            app.add_plugins(bevy::dev_tools::fps_overlay::FpsOverlayPlugin::default());
        }

        if self.flags.contains(Flags::DEBUG_INSPECTOR) {
            app.add_plugins((
                bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
                bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
            ));
        }

        // TODO update to work with new unavi-space crate
        // if self.flags.contains(Flags::DEBUG_NETWORK) {
        //     app.init_resource::<network_stats::NetworkStats>()
        //         .add_systems(
        //             FixedUpdate,
        //             (
        //                 network_stats::collect_network_events,
        //                 network_stats::update_bandwidth_stats,
        //                 network_stats::update_tickrate_stats,
        //                 network_stats::detect_dropped_frames,
        //             ),
        //         )
        //         .add_systems(Startup, network_ui::spawn_devtools_overlay)
        //         .add_systems(Update, network_ui::update_network_stats_text);
        // }

        if self.flags.contains(Flags::DEBUG_PHYSICS) {
            app.add_plugins(avian3d::debug_render::PhysicsDebugPlugin);
        }
    }
}
