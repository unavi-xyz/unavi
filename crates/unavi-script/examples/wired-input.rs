use bevy::{
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_wds::{LocalActor, LocalBlobs, util::create_test_wds};
use unavi_script::{
    ScriptPermissions, SpawnLocalScript, load::local::ScriptSource, permissions::ApiName,
};

mod util;

const SCRIPT_PATH: &str = "wasm/example/wired_input.wasm";

fn main() {
    util::copy_assets_to_project_dir(&[SCRIPT_PATH]);

    let (actor, blobs) = create_test_wds();

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(AssetPlugin {
                file_path: util::assets_dir().to_string_lossy().to_string(),
                ..Default::default()
            })
            .set(LogPlugin {
                filter: format!("{DEFAULT_FILTER},loro_internal=off"),
                ..Default::default()
            }),
        PanOrbitCameraPlugin,
        bevy_hsd::HsdPlugin,
        bevy_wds::WdsPlugin,
        unavi_input::InputPlugin,
        unavi_script::ScriptPlugin,
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
        Transform::from_xyz(3.0, 8.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let mut permissions = ScriptPermissions::default();
    permissions.api.insert(ApiName::SystemInput);

    commands.trigger(SpawnLocalScript {
        permissions,
        source: ScriptSource::Path(SCRIPT_PATH.to_string()),
    });
}
