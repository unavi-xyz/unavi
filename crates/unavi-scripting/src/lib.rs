use bevy::prelude::*;

use self::{asset::Wasm, load::Scripts};

pub mod asset;
mod execution;
mod host;
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
                    host::wired_gltf::handler::handle_wired_gltf_actions,
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

        let path = format!(
            "components/{}/{}/{}.wasm",
            env!("CARGO_PKG_VERSION"),
            namespace,
            package
        );
        let wasm = asset_server.load(path);

        Self {
            name: name.into(),
            wasm,
        }
    }
}
