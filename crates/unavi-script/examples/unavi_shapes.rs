use bevy::{
    log::{
        DEFAULT_FILTER,
        LogPlugin,
    },
    prelude::*,
};
use bevy_hsd::load::{
    LoadHsd,
    on_load_spawn_doc,
};
use bevy_panorbit_camera::{
    PanOrbitCamera,
    PanOrbitCameraPlugin,
};
use bevy_wds::{
    LocalActor,
    LocalBlobs,
    LocalDocs,
};
use unavi_script::permissions::ApiPermissions;

use crate::util::create_test_wds;

mod util;

const SCRIPT_PATH: &str = "hsd/example_unavi_shapes.hsd";

fn main() {
    let (actor, docs, blobs) = create_test_wds();

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(AssetPlugin {
                file_path: "../unavi-client/assets/".to_string(),
                ..Default::default()
            })
            .set(LogPlugin {
                filter: format!("{DEFAULT_FILTER},loro_internal=off"),
                ..Default::default()
            }),
        PanOrbitCameraPlugin,
        bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
        bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        bevy_hsd::HsdPlugin,
        bevy_iroh::IrohPlugin,
        bevy_wds::WdsPlugin,
        unavi_util::UtilPlugin,
        unavi_script::ScriptPlugin,
    ))
    .add_systems(Startup, init_scene);

    app.world_mut()
        .spawn((LocalActor(actor), LocalDocs(docs), LocalBlobs(blobs)));

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
            extra: None,
            on_load: Some(Box::new(on_load_spawn_doc)),
        },
        ApiPermissions::default(),
    ));
}
