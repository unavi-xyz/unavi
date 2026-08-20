use avian3d::debug_render::PhysicsDebugPlugin;
use bevy::{
    dev_tools::fps_overlay::{
        FpsOverlayConfig,
        FpsOverlayPlugin,
        FrameTimeGraphConfig,
    },
    prelude::*,
};
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin,
    quick::WorldInspectorPlugin,
};
use unavi_devtools::DevToolsPlugin;

mod bevy_panel;
mod event_gizmos;

/// Client-side dev tools: the shared overlay plus a "Bevy" panel toggling the
/// engine debug views (FPS, world inspector, physics and event gizmos).
pub struct ClientDevToolsPlugin;

impl Plugin for ClientDevToolsPlugin {
    fn build(&self, app: &mut App) {
        event_gizmos::install_emit_observer();

        app.add_plugins((
            DevToolsPlugin,
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    enabled: false,
                    frame_time_graph_config: FrameTimeGraphConfig {
                        enabled: false,
                        ..default()
                    },
                    ..default()
                },
            },
            PhysicsDebugPlugin,
            EguiPlugin::default(),
            WorldInspectorPlugin::default()
                .run_if(bevy_panel::toggled(bevy_panel::Toggle::Inspector)),
        ))
        .init_resource::<bevy_panel::DevToggles>()
        .init_resource::<event_gizmos::EventPings>()
        .add_systems(Startup, bevy_panel::spawn)
        .add_observer(bevy_panel::on_toggle)
        .add_systems(
            Update,
            (
                bevy_panel::apply_toggles,
                bevy_panel::apply_fps_display,
                (
                    event_gizmos::update_event_pings,
                    event_gizmos::draw_receptors,
                    event_gizmos::draw_event_pings,
                )
                    .run_if(bevy_panel::toggled(bevy_panel::Toggle::Events)),
            ),
        );
    }
}
