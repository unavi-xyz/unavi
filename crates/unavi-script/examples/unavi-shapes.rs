use bevy::prelude::*;
use unavi_script::{LoadScriptAsset, ScriptPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: "../unavi/assets".to_string(),
                ..Default::default()
            }),
            ScriptPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut events: EventWriter<LoadScriptAsset>) {
    events.write(LoadScriptAsset {
        namespace: "example",
        package: "shapes",
    });
}
