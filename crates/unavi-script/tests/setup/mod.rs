use std::{
    fmt::Display,
    sync::{Arc, LazyLock, Mutex},
    time::Duration,
};

use bevy::{
    log::{BoxedLayer, LogPlugin},
    prelude::*,
};
use tracing::{
    Event, Subscriber,
    span::{Attributes, Id},
};
use tracing_subscriber::{
    Layer,
    field::MakeVisitor,
    fmt::format::{PrettyFields, Writer},
    layer::Context,
    registry::LookupSpan,
};
use unavi_script::{LoadScriptAsset, ScriptPlugin};

pub mod logs;

const TICK: Duration = Duration::from_millis(200);

pub fn setup_test_app(package: &'static str) -> App {
    let mut app = App::new();

    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: "../unavi/assets".to_string(),
            ..Default::default()
        },
        LogPlugin {
            custom_layer: logs::custom_layer,
            ..Default::default()
        },
        ScriptPlugin,
    ))
    .insert_resource(Time::<Virtual>::from_max_delta(TICK))
    .insert_resource(Time::<Fixed>::from_duration(TICK))
    .add_systems(Startup, move |mut events: EventWriter<LoadScriptAsset>| {
        events.write(LoadScriptAsset {
            namespace: "test",
            package,
        });
    });

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
}

pub fn tick_app(app: &mut App) {
    app.update();
    std::thread::sleep(TICK);
    std::thread::sleep(TICK);
}
