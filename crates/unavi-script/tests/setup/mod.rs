use std::time::Duration;

use bevy::{log::LogPlugin, prelude::*};
use bevy_hsd::HsdPlugin;
use bevy_wds::{LocalActor, LocalBlobs, WdsPlugin, util::create_test_wds};
use tracing_subscriber::Layer;
use unavi_script::{ScriptPermissions, ScriptPlugin, SpawnLocalScript, load::local::ScriptSource};

use crate::setup::logs::LOGS;

pub mod logs;

const TICK: Duration = Duration::from_millis(100);

pub fn setup_test_app(package: &'static str, wasm_override: Option<Vec<u8>>) -> App {
    let (actor, blobs) = create_test_wds();

    let source = wasm_override.map_or_else(
        || ScriptSource::Path(format!("wasm/test/{package}.wasm")),
        ScriptSource::Bytes,
    );

    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: "../unavi-client/assets".to_string(),
            ..Default::default()
        },
        LogPlugin {
            custom_layer: |_| Some(LOGS.clone().boxed()),
            ..Default::default()
        },
        WdsPlugin,
        HsdPlugin,
        ScriptPlugin,
    ))
    .insert_resource(Time::<Virtual>::from_max_delta(TICK))
    .insert_resource(Time::<Fixed>::from_duration(TICK));

    app.world_mut()
        .spawn((LocalActor(actor), LocalBlobs(blobs)));

    app.world_mut().trigger(SpawnLocalScript {
        permissions: ScriptPermissions::default(),
        source,
    });

    app
}

pub fn construct_script(app: &mut App) {
    // Ticks 1-2: load wasm asset, upload to WDS, spawn HSD doc.
    tick_app(app);
    tick_app(app);

    // Tick 3: init_script_overlay + init_hsd_doc run (chained).
    tick_app(app);
    tick_app(app);

    // Ticks 5-7: load_hsd_scripts fires, spawns blob fetch task, polls.
    tick_app(app);
    tick_app(app);
    tick_app(app);

    // Ticks 8-10: load_scripts instantiates wasm component.
    tick_app(app);
    tick_app(app);
    tick_app(app);

    // Ticks 11-12: begin_init_scripts / end_init_scripts.
    tick_app(app);
    tick_app(app);
}

pub fn tick_app(app: &mut App) {
    app.update();
    // Sleep to allow async work to run.
    std::thread::sleep(Duration::from_millis(300));
}
