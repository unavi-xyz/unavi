use bevy::{
    log::{
        DEFAULT_FILTER,
        LogPlugin,
    },
    prelude::*,
};
use bevy_hsd::load::LoadHsd;
use bevy_iroh::store::{
    LocalBlobStore,
    LocalBlobs,
    LocalStore,
};
use bevy_panorbit_camera::{
    PanOrbitCamera,
    PanOrbitCameraPlugin,
};
use unavi_policy::document::DocumentPolicy;

use crate::util::create_test_store;

mod util;

const SCRIPT_PATH: &str = "hsd/example_unavi_shapes.hsdz";

fn main() {
    let store = create_test_store();

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(AssetPlugin {
                file_path: "../unavi-client/assets/".to_string(),
                ..Default::default()
            })
            .set(LogPlugin {
                filter: DEFAULT_FILTER.to_string(),
                ..Default::default()
            }),
        PanOrbitCameraPlugin,
        bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
        bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        bevy_hsd::HsdPlugin,
        bevy_iroh::IrohPlugin,
        unavi_util::UtilPlugin,
        unavi_script::ScriptPlugin,
    ))
    .add_systems(Startup, init_scene);

    app.world_mut().spawn((
        LocalBlobStore(store.store.blob_store().clone()),
        LocalBlobs(store.store.blobs().clone()),
        LocalStore(store.store.clone()),
    ));

    app.run();
}

fn init_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(5.0, 8.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        PanOrbitCamera::default(),
        Transform::from_xyz(3.0, 8.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let handle = asset_server.load(SCRIPT_PATH);
    commands.spawn((
        LoadHsd {
            handle,
            on_load: None,
        },
        DocumentPolicy::system(),
    ));
}
