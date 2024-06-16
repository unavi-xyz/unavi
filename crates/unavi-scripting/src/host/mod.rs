use anyhow::Result;
use wasm_bridge::component::Linker;

// use self::wired_gltf::{WiredGltfData, WiredGltfReceiver};

use super::StoreState;

pub mod wired_gltf;
mod wired_log;

pub fn add_host_script_apis(linker: &mut Linker<StoreState>) -> Result<()> {
    wired_gltf::add_to_host(linker)?;
    wired_log::add_to_host(linker)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use tracing_test::traced_test;

    use crate::{
        asset::{Wasm, WasmLoader},
        load::{LoadedScript, Scripts},
        ScriptBundle,
    };

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
        .init_non_send_resource::<Scripts>();

        app.add_systems(
            Update,
            (
                crate::load::load_scripts,
                // crate::host::wired_gltf::query::query_node_data,
                crate::execution::init_scripts,
                crate::execution::update_scripts,
                // super::wired_gltf::handler::handle_wired_gltf_actions,
            )
                .chain(),
        );

        let asset_server = app.world.get_resource::<AssetServer>().unwrap();
        app.world.spawn(ScriptBundle::load(name, asset_server));

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
        test_script("test:wired-gltf").await;
        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));
        assert!(logs_contain("Called script init"));
        assert!(logs_contain("Called script update"));
    }

    #[tokio::test]
    #[traced_test]
    async fn test_example_wired_gltf() {
        test_script("example:wired-gltf").await;
        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));
    }
}
