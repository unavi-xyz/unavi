use std::{
    path::PathBuf,
    sync::LazyLock,
};

use avian3d::prelude::*;
use bevy::{
    camera::visibility::RenderLayers,
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
use bevy_vrm::first_person::{
    DEFAULT_RENDER_LAYERS,
    FirstPersonFlag,
};
use bevy_wds::{
    LocalActor,
    LocalBlobs,
    LocalDocs,
};
use directories::ProjectDirs;
use unavi_agent::LocalAgent;
use unavi_script::permissions::{
    ApiName,
    ApiPermissions,
};

use crate::util::create_test_wds;

mod util;

const SCRIPT_PATH: &str = "hsd/example_wired_agent.hsd";

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-client").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    dirs
});

fn main() -> anyhow::Result<()> {
    let assets_path = "../unavi-client/assets/".to_string();

    // Copy runtime assets (VRM and glb animations) to assets dir.
    let src = DIRS.data_local_dir().join("assets");
    let dst = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(&assets_path)
        .canonicalize()?;
    std::fs::create_dir_all(dst.join("model"))?;
    for path in ["model/animations.glb", "model/default.vrm"] {
        let src = src.join(path);
        let dst = dst.join(path);
        println!(
            "Copying {} -> {}",
            src.to_string_lossy(),
            dst.to_string_lossy()
        );
        std::fs::copy(src, dst)?;
    }

    let (actor, docs, blobs) = create_test_wds();

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(AssetPlugin {
                file_path: assets_path,
                ..Default::default()
            })
            .set(LogPlugin {
                filter: format!("{DEFAULT_FILTER},loro_internal=off"),
                ..Default::default()
            }),
        PanOrbitCameraPlugin,
        avian3d::PhysicsPlugins::default(),
        bevy_inspector_egui::bevy_egui::EguiPlugin::default(),
        bevy_inspector_egui::quick::WorldInspectorPlugin::default(),
        bevy_hsd::HsdPlugin,
        bevy_iroh::IrohPlugin,
        bevy_wds::WdsPlugin,
        unavi_util::UtilPlugin,
        unavi_agent::AgentPlugin,
        unavi_avatar::AvatarPlugin,
        unavi_script::ScriptPlugin,
    ))
    .add_observer(on_agent_load)
    .add_systems(Startup, init_scene);

    app.world_mut()
        .spawn((LocalActor(actor), LocalDocs(docs), LocalBlobs(blobs)));

    app.run();

    Ok(())
}

fn init_scene(mut commands: Commands) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(5.0, 8.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Transform::from_xyz(0.0, -2.0, 0.0),
        Collider::cuboid(4.0, 0.5, 4.0),
    ));

    commands.spawn(LocalAgent);
}

fn on_agent_load(
    _: On<Add, Camera3d>,
    mut cameras: Query<&mut Camera>,
    mut commands: Commands,
    mut added: Local<bool>,
    asset_server: Res<AssetServer>,
) {
    if *added {
        return;
    }
    *added = true;

    info!("Local agent loaded, spawning script");

    let mut cam = cameras.single_mut().expect("single camera");
    cam.is_active = false;

    commands.spawn((
        PanOrbitCamera::default(),
        Transform::from_xyz(-3.0, 5.0, -6.0).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::from_layers(&[0])
            .union(&DEFAULT_RENDER_LAYERS[&FirstPersonFlag::ThirdPersonOnly]),
    ));

    let handle = asset_server.load(SCRIPT_PATH);
    commands.spawn((
        LoadHsd {
            handle,
            extra: None,
            on_load: Some(Box::new(on_load_spawn_doc)),
        },
        ApiPermissions::default().with(ApiName::LocalAgent),
    ));
}
