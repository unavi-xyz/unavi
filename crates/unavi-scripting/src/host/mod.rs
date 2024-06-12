use anyhow::Result;
use wasm_component_layer::{Linker, Store};

use self::wired_gltf::{WiredGltfData, WiredGltfReceiver};

use super::{load::EngineBackend, StoreData};

pub mod wired_gltf;
mod wired_log;

pub struct HostScriptResults {
    pub wired_gltf_components: (WiredGltfReceiver, WiredGltfData),
}

pub fn add_host_script_apis(
    store: &mut Store<StoreData, EngineBackend>,
    linker: &mut Linker,
) -> Result<HostScriptResults> {
    let wired_gltf_components = wired_gltf::add_to_host(store, linker)?;
    wired_log::add_to_host(store, linker)?;

    Ok(HostScriptResults {
        wired_gltf_components,
    })
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use tracing_test::traced_test;

    use crate::{
        asset::{Wasm, WasmLoader},
        load::{LoadedScript, WasmStores},
        ScriptBundle,
    };

    fn load_component_wasm(asset_server: &AssetServer, name: &str) -> Handle<Wasm> {
        let path = format!("components/{}_{}.wasm", name, env!("CARGO_PKG_VERSION"));
        asset_server.load(path)
    }

    pub async fn test_script(name: &str) {
        let mut app = App::new();

        app.add_plugins((
            MinimalPlugins,
            AssetPlugin {
                file_path: "../unavi-app/assets".to_string(),
                ..Default::default()
            },
        ))
        .init_asset::<Mesh>()
        .init_asset::<StandardMaterial>()
        .init_asset::<Wasm>()
        .init_asset_loader::<WasmLoader>()
        .init_non_send_resource::<WasmStores>();

        app.add_systems(
            Update,
            (
                crate::load::load_scripts,
                crate::host::wired_gltf::query::query_node_data,
                crate::execution::init_scripts,
                crate::execution::update_scripts,
                super::wired_gltf::handler::handle_wired_gltf_actions,
            )
                .chain(),
        );

        let asset_server = app.world.get_resource_mut::<AssetServer>().unwrap();
        let wasm = load_component_wasm(&asset_server, name);

        app.world.spawn(ScriptBundle {
            name: name.into(),
            wasm,
        });

        for _ in 0..5 {
            tokio::time::sleep(Duration::from_secs_f32(0.2)).await;
            app.update();
        }

        let mut loaded_scripts = app.world.query::<(Entity, &LoadedScript)>();
        let len = loaded_scripts.iter(&app.world).len();
        assert_eq!(len, 1);
    }

    #[tokio::test]
    #[traced_test]
    async fn test_wired_gltf() {
        test_script("test_wired_gltf").await;
        assert!(logs_contain("Hello from script!"));
        assert!(!logs_contain("error"));
        assert!(!logs_contain("ERROR"));
    }

    #[tokio::test]
    #[traced_test]
    async fn test_example_wired_gltf() {
        test_script("example_wired_gltf").await;
        assert!(!logs_contain("error"));
        assert!(!logs_contain("ERROR"));
    }
}
