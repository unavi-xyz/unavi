use bevy::prelude::*;

mod init;
mod instantiate;
mod render;
mod tick;

pub struct WebEnginePlugin;

impl Plugin for WebEnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render::render_tick_scripts)
            .add_systems(
                FixedUpdate,
                (
                    instantiate::instantiate_scripts,
                    instantiate::poll_instantiating,
                    init::init_scripts,
                    init::poll_initing_scripts,
                    tick::tick_scripts,
                ),
            );
    }
}
