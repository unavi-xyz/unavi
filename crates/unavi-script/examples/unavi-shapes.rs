use bevy::prelude::*;
use bevy_wds::{LocalActor, LocalBlobs, WdsPlugin, util::create_test_wds};
use unavi_script::{LoadScriptAsset, ScriptPlugin};

fn main() {
    let (actor, blobs) = create_test_wds();

    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(AssetPlugin {
            file_path: "../unavi-client/assets".to_string(),
            ..Default::default()
        }),
        WdsPlugin,
        ScriptPlugin,
    ))
    .add_systems(Startup, load_script);

    app.world_mut().spawn((LocalActor(actor), LocalBlobs(blobs)));

    app.run();
}

fn load_script(mut events: MessageWriter<LoadScriptAsset>) {
    events.write(LoadScriptAsset {
        namespace: "example",
        package: "shapes",
    });
}
