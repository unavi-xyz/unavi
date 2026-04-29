use bevy::prelude::*;

mod instantiate;
mod js_runtime;

pub struct WebEnginePlugin;

impl Plugin for WebEnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, instantiate::instantiate_scripts);
    }
}
