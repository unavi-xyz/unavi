use std::time::Duration;

use bevy::{log::LogPlugin, prelude::*};
use tracing_subscriber::Layer;
use unavi_script::{LoadScriptAsset, ScriptPlugin};

use crate::setup::logs::LOGS;

pub mod logs;

const TICK: Duration = Duration::from_millis(100);

pub fn setup_test_app(package: &'static str) -> App {
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
        ScriptPlugin,
    ))
    .insert_resource(Time::<Virtual>::from_max_delta(TICK))
    .insert_resource(Time::<Fixed>::from_duration(TICK))
    .add_systems(
        Startup,
        move |mut events: MessageWriter<LoadScriptAsset>| {
            events.write(LoadScriptAsset {
                namespace: "test",
                package,
            });
        },
    );

    app
}

pub fn construct_script(app: &mut App) {
    // Load script asset.
    tick_app(app);
    tick_app(app);

    // Instantiate wasm.
    tick_app(app);
    tick_app(app);

    // Execute script constructor.
    tick_app(app);
    tick_app(app);

    // Init script cycle.
    tick_app(app);
}

pub fn tick_app(app: &mut App) {
    app.update();
    std::thread::sleep(2 * TICK);
}
