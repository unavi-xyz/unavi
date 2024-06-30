use anyhow::Result;
use wasm_bridge::component::Linker;

use super::state::StoreState;

pub mod wired_gltf;
mod wired_input;
mod wired_log;
pub mod wired_physics;

pub fn add_host_apis(linker: &mut Linker<StoreState>) -> Result<()> {
    wired_gltf::add_to_linker(linker)?;
    wired_input::add_to_linker(linker)?;
    wired_log::add_to_linker(linker)?;
    wired_physics::add_to_linker(linker)?;
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

        app.add_systems(Update, crate::load::load_scripts);

        let asset_server = app.world.get_resource::<AssetServer>().unwrap();
        let entity = app.world.spawn(ScriptBundle::load(name, asset_server)).id();

        let mut did_load = false;

        for _ in 0..10 {
            tokio::time::sleep(Duration::from_secs_f32(1.0)).await;
            app.update();

            let loaded_script = app.world.get::<LoadedScript>(entity);
            if loaded_script.is_some() {
                did_load = true;
                break;
            };
        }

        assert!(did_load);

        app.add_systems(
            Update,
            (
                crate::execution::init_scripts,
                crate::execution::update_scripts,
                crate::actions::handler::handle_actions,
            )
                .chain(),
        );

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

    #[tokio::test]
    #[traced_test]
    async fn example_wired_input() {
        test_script("example:wired-input").await;
        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));
    }

    #[tokio::test]
    #[traced_test]
    async fn example_wired_physics() {
        test_script("example:wired-physics").await;
        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));
    }
}
