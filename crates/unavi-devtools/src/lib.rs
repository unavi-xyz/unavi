use bevy::{
    feathers::{
        FeathersPlugins,
        dark_theme::create_dark_theme,
        theme::UiTheme,
    },
    prelude::*,
};

pub mod channel;
pub mod overlay;
pub mod scroll;
pub mod tabs;

/// Installs the dev tools overlay: a tabbed, `~`-toggled surface that other
/// crates contribute panels to by spawning a [`tabs::DevPanel`] entity.
pub struct DevToolsPlugin;

impl Plugin for DevToolsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FeathersPlugins)
            .insert_resource(UiTheme(create_dark_theme()))
            .init_resource::<overlay::DevToolsActive>()
            .init_resource::<tabs::ActiveDevPanel>()
            .add_systems(PreStartup, overlay::spawn_overlay)
            .add_systems(
                Update,
                (
                    overlay::toggle_overlay,
                    tabs::apply_active_panel,
                    tabs::highlight_active_tab,
                    scroll::apply_wheel_scroll
                        .run_if(|active: Res<overlay::DevToolsActive>| active.0),
                ),
            )
            .add_observer(tabs::register_panel)
            .add_observer(tabs::activate_panel);
    }
}
