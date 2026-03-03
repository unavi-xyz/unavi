use bevy::prelude::*;
use bevy_hsd::HsdPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_wds::{LocalActor, LocalBlobs, WdsPlugin, util::create_test_wds};
use unavi_script::{
    ScriptPlugin,
    dev::{WasmDevPlugin, WasmSource},
};

fn main() {
    let (actor, blobs) = create_test_wds();

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(AssetPlugin {
            file_path: "../unavi-client/assets".to_string(),
            ..Default::default()
        }),
        PanOrbitCameraPlugin,
        WdsPlugin,
        HsdPlugin,
        ScriptPlugin,
        WasmDevPlugin(WasmSource::Path("wasm/example/shapes.wasm".to_string())),
    ))
    .add_systems(Startup, init_scene);

    app.world_mut()
        .spawn((LocalActor(actor), LocalBlobs(blobs)));

    app.run();
}

fn init_scene(mut commands: Commands) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(5.0, 8.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        PanOrbitCamera::default(),
        Transform::from_xyz(3.0, 3.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
