use bevy::prelude::*;
use unavi_constants::assets::WASM_ASSETS_DIR;

use self::{asset::Wasm, load::Scripts};

pub mod api;
pub mod asset;
mod execution;
mod load;
mod state;

mod wired_script {
    wasm_bridge::component::bindgen!({
        path: "../../wired-protocol/spatial/wit/wired-script",
        async: true,
    });
}

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<Wasm>()
            .init_non_send_resource::<Scripts>()
            .add_systems(
                FixedUpdate,
                (
                    (execution::init_scripts, execution::update_scripts).chain(),
                    load::load_scripts,
                ),
            );
    }
}

#[derive(Bundle)]
pub struct ScriptBundle {
    pub name: Name,
    pub wasm: Handle<Wasm>,
}

impl ScriptBundle {
    /// Loads a give WASM script from the assets folder, in the `namespace:package` format.
    pub fn load(name: &str, asset_server: &AssetServer) -> Self {
        let (namespace, package) = name.split_once(':').unwrap();

        let path = format!("{}/{}/{}.wasm", WASM_ASSETS_DIR, namespace, package);
        let wasm = asset_server.load(path);

        Self {
            name: name.into(),
            wasm,
        }
    }
}
