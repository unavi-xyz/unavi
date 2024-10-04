use bevy::prelude::*;
use execution::ScriptTickrate;
use load::{DefaultMaterial, ScriptExecutionLevel};
use unavi_constants::assets::WASM_ASSETS_DIR;

use self::load::ScriptMap;

pub mod api;
mod asset;
mod data;
mod env;
pub mod execution;
pub mod load;
mod raycast;

pub use asset::*;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        let mut materials = app
            .world_mut()
            .get_resource_mut::<Assets<StandardMaterial>>()
            .unwrap();
        let default_material = materials.add(StandardMaterial::default());

        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<Wasm>()
            .init_non_send_resource::<ScriptMap>()
            .insert_resource(DefaultMaterial(default_material))
            .add_systems(Update, raycast::handle_raycast_input)
            .add_systems(
                FixedUpdate,
                (
                    load::load_scripts,
                    execution::tick_scripts,
                    (
                        api::wired::physics::systems::update_physics_transforms,
                        api::wired::player::systems::copy_global_transforms,
                        api::wired::player::systems::copy_transforms,
                        api::wired::player::systems::update_player_skeletons,
                    ),
                    execution::init_scripts,
                    execution::update_scripts,
                )
                    .chain(),
            );
    }
}

#[derive(Bundle)]
pub struct ScriptBundle {
    pub execution_level: ScriptExecutionLevel,
    pub name: Name,
    pub tickrate: ScriptTickrate,
    pub wasm: Handle<Wasm>,
}

impl ScriptBundle {
    /// Loads a give WASM script from the assets folder, in the `namespace:package` format.
    pub fn load(name: &str, asset_server: &AssetServer) -> Self {
        let (namespace, package) = name.split_once(':').expect("Script name has no colon");

        let path = format!("{}/{}/{}.wasm", WASM_ASSETS_DIR, namespace, package);
        let wasm = asset_server.load(path);

        Self {
            execution_level: ScriptExecutionLevel::World,
            name: name.into(),
            tickrate: ScriptTickrate::default(),
            wasm,
        }
    }
}
