use bevy::prelude::*;

pub mod asset;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<asset::Wasm>();
        // .add_systems(OnEnter(AppState::InWorld), setup_runtimes)
        // .add_systems(
        //     Update,
        //     (
        //         scripts::instantiate_scripts,
        //         scripts::set_runtime_name,
        //         lifecycle::init_scripts,
        //         lifecycle::update_scripts,
        //         lifecycle::exit_scripts,
        //     )
        //         .chain(),
        // );
    }
}
