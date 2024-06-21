use anyhow::Result;
use wasm_bridge::component::Linker;

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

    const UPDATES: usize = 4;

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
                crate::execution::update_scripts,
                crate::execution::init_scripts,
                super::wired_gltf::handler::handle_wired_gltf_actions,
            )
                .chain(),
        );

        let asset_server = app.world.get_resource::<AssetServer>().unwrap();
        let entity = app.world.spawn(ScriptBundle::load(name, asset_server)).id();

        for _ in 0..3 {
            tokio::time::sleep(Duration::from_secs_f32(0.5)).await;
            app.update();
        }

        let loaded_script = app.world.get::<LoadedScript>(entity);
        assert!(loaded_script.is_some());

        for _ in 0..UPDATES {
            tokio::time::sleep(Duration::from_secs_f32(0.1)).await;
            app.update();
        }
    }

    #[tokio::test]
    #[traced_test]
    async fn test_wired_gltf() {
        test_script("test:wired-gltf").await;
        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));

        logs_assert(|logs| {
            let mut found_constructs = 0;
            let mut found_updates = 0;

            for log in logs {
                if log.contains("Called script construct") {
                    found_constructs += 1;
                }

                if log.contains("Called script update") {
                    found_updates += 1;
                }
            }

            assert_eq!(found_constructs, 1);
            assert_eq!(found_updates, UPDATES);

            Ok(())
        });
    }

    #[tokio::test]
    #[traced_test]
    async fn example_wired_gltf() {
        test_script("example:wired-gltf").await;
        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));
    }
}
