use std::time::Duration;

use avian3d::PhysicsPlugins;
use bevy::{gizmos::GizmoPlugin, input::InputPlugin, prelude::*, scene::ScenePlugin};
use tracing_test::traced_test;

use unavi_scripting::{load::LoadedScript, ScriptBundle, ScriptingPlugin};
use unavi_world::util::init_user_actor;

const NUM_UPDATES: usize = 4;

pub async fn test_script(name: &str) {
    let mut app = App::new();
    init_user_actor(app.world_mut()).await;

    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: "../unavi-app/assets".to_string(),
            ..Default::default()
        },
        TransformPlugin,
        HierarchyPlugin,
        ScenePlugin,
    ))
    .init_asset::<Mesh>()
    .init_asset::<Shader>()
    .init_asset::<StandardMaterial>()
    .add_plugins((
        GizmoPlugin,
        InputPlugin,
        PhysicsPlugins::default(),
        ScriptingPlugin,
    ));

    let world = app.world_mut();
    let asset_server = world.get_resource::<AssetServer>().unwrap();
    let entity = world.spawn(ScriptBundle::load(name, asset_server)).id();

    let mut did_load = false;

    let step = Duration::from_secs_f32(0.25);
    app.insert_resource(Time::<Fixed>::from_duration(step));

    for i in 0..10 {
        debug!("Loading... ({})", i);

        tokio::time::sleep(step).await;
        app.update();

        let loaded_script = app.world().get::<LoadedScript>(entity);
        if loaded_script.is_some() {
            did_load = true;
            break;
        };
    }

    assert!(did_load);

    for i in 0..NUM_UPDATES {
        debug!("Updating... ({})", i);
        tokio::time::sleep(step).await;
        app.update();
    }
}

// #[tokio::test]
// #[traced_test]
// async fn test_wired_dwn() {
//     test_script("test:wired-dwn").await;
//     assert!(!logs_contain("ERROR"));
//     assert!(!logs_contain("error"));
//     assert!(!logs_contain("WARN"));
//     assert!(!logs_contain("warn"));
// }

#[tokio::test]
#[traced_test]
async fn test_wired_physics() {
    test_script("test:wired-physics").await;
    assert!(!logs_contain("ERROR"));
    assert!(!logs_contain("error"));
    assert!(!logs_contain("WARN"));
    assert!(!logs_contain("warn"));
}

#[tokio::test]
#[traced_test]
async fn test_wired_scene() {
    test_script("test:wired-scene").await;
    assert!(!logs_contain("ERROR"));
    assert!(!logs_contain("error"));
    assert!(!logs_contain("WARN"));
    assert!(!logs_contain("warn"));

    // TODO: Move this to an independent test
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
        assert_eq!(found_updates, NUM_UPDATES);

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
async fn example_wired_player() {
    test_script("example:wired-player").await;
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
async fn example_unavi_layout() {
    test_script("example:unavi-layout").await;
    assert!(!logs_contain("ERROR"));
    assert!(!logs_contain("error"));
    assert!(!logs_contain("WARN"));
    assert!(!logs_contain("warn"));
}

#[tokio::test]
#[traced_test]
async fn example_unavi_shapes() {
    test_script("example:unavi-shapes").await;
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
