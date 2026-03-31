use std::time::{Duration, Instant};

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
    .init_asset::<StandardMaterial>()
    .init_asset::<Mesh>()
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

/// Poll `app` until `condition()` returns true or `timeout` elapses.
///
/// Returns `true` if the condition was met, `false` on timeout.
pub fn wait_until(app: &mut App, condition: impl Fn() -> bool, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        tick_app(app);
        if condition() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
    }
}

pub fn tick_app(app: &mut App) {
    app.update();
    // Sleep to allow async work to run and for virtual time to advance by TICK.
    std::thread::sleep(Duration::from_millis(300));
}
