use bevy::prelude::*;

use self::{
    asset::Wasm,
    script::{ScriptLoadQueue, WasmEngine},
};

mod asset;
mod script;
mod stream;

pub struct ScriptingPlugin;

impl Plugin for ScriptingPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(asset::WasmLoader)
            .init_asset::<Wasm>()
            .init_resource::<ScriptLoadQueue>()
            .init_resource::<WasmEngine>()
            .add_systems(Startup, load_unavi_system);

        #[cfg(target_family = "wasm")]
        {
            use self::script::wasm::{load_scripts, update_scripts, Scripts};

            app.init_non_send_resource::<Scripts>()
                .add_systems(Update, (load_scripts, update_scripts));
        }

        #[cfg(not(target_family = "wasm"))]
        {
            use self::script::native::{load_scripts, update_scripts, Scripts};

            app.init_resource::<Scripts>()
                .add_systems(Update, (load_scripts, update_scripts));
        }
    }
}

fn load_unavi_system(mut load_queue: ResMut<ScriptLoadQueue>, asset_server: Res<AssetServer>) {
    let handle = asset_server.load("components/unavi_system.wasm");
    load_queue.0.push(handle);
}
