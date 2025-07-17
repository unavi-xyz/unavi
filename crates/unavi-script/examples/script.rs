use bevy::prelude::*;
use unavi_script::ScriptPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: "../unavi/assets".to_string(),
                ..Default::default()
            }),
            ScriptPlugin,
        ))
        .run();
}
