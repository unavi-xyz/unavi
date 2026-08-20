use bevy::prelude::*;

mod fixed_update;
mod init;
mod instantiate;
mod log;
mod update;

pub struct WebEnginePlugin;

impl Plugin for WebEnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update::update_scripts).add_systems(
            FixedUpdate,
            (
                instantiate::instantiate_scripts,
                instantiate::poll_instantiating,
                init::init_scripts,
                init::poll_initing_scripts,
                fixed_update::fixed_update_scripts,
            ),
        );
    }
}
