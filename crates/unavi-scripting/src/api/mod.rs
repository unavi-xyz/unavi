use anyhow::Result;
use wasm_bridge::component::Linker;

use super::state::StoreState;

pub(crate) mod utils;
pub mod wired;

pub(crate) fn add_host_apis(linker: &mut Linker<StoreState>) -> Result<()> {
    wired::scene::add_to_linker(linker)?;
    wired::input::add_to_linker(linker)?;
    wired::log::add_to_linker(linker)?;
    wired::physics::add_to_linker(linker)?;
    wired::player::add_to_linker(linker)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use tracing_test::traced_test;

    use crate::{
        asset::{Wasm, WasmLoader},
        load::{DefaultMaterial, LoadedScript, ScriptMap},
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
        .init_non_send_resource::<ScriptMap>()
        .insert_resource(DefaultMaterial(Handle::default()));

        app.add_systems(Update, crate::load::load_scripts);

        let world = app.world_mut();
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let entity = world.spawn(ScriptBundle::load(name, asset_server)).id();

        let mut did_load = false;

        for _ in 0..10 {
            tokio::time::sleep(Duration::from_secs_f32(1.0)).await;
            app.update();

            let loaded_script = app.world().get::<LoadedScript>(entity);
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
    async fn test_wired_scene() {
        test_script("test:wired-scene").await;
        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));
        assert!(!logs_contain("WARN"));
        assert!(!logs_contain("warn"));

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
            assert_eq!(found_updates, 1);

            Ok(())
        });
    }

    #[tokio::test]
    #[traced_test]
    async fn example_wired_scene() {
        test_script("example:wired-scene").await;
        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));
        assert!(!logs_contain("WARN"));
        assert!(!logs_contain("warn"));
    }

    #[tokio::test]
    #[traced_test]
    async fn example_wired_input() {
        test_script("example:wired-input").await;
        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));
        assert!(!logs_contain("WARN"));
        assert!(!logs_contain("warn"));
    }

    #[tokio::test]
    #[traced_test]
    async fn example_wired_physics() {
        test_script("example:wired-physics").await;
        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));
        assert!(!logs_contain("WARN"));
        assert!(!logs_contain("warn"));
    }

    #[tokio::test]
    #[traced_test]
    async fn example_unavi_scene() {
        test_script("example:unavi-scene").await;
        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));
        assert!(!logs_contain("WARN"));
        assert!(!logs_contain("warn"));
    }

    #[tokio::test]
    #[traced_test]
    async fn example_unavi_ui() {
        test_script("example:unavi-ui").await;
        assert!(!logs_contain("ERROR"));
        assert!(!logs_contain("error"));
        assert!(!logs_contain("WARN"));
        assert!(!logs_contain("warn"));
    }
}
