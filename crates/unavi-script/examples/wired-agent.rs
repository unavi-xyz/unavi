use std::{path::PathBuf, sync::LazyLock};

use avian3d::prelude::Collider;
use bevy::{
    camera::visibility::RenderLayers,
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_vrm::first_person::{DEFAULT_RENDER_LAYERS, FirstPersonFlag};
use bevy_wds::{LocalActor, LocalBlobs, util::create_test_wds};
use directories::ProjectDirs;
use unavi_agent::LocalAgent;
use unavi_script::{ScriptPermissions, SpawnLocalScript, load::local::ScriptSource};

pub static DIRS: LazyLock<directories::ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-client").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    dirs
});

pub fn assets_dir() -> PathBuf {
    DIRS.data_local_dir().join("assets")
}

const SCRIPT_PATH: &str = "wasm/example/wired_agent.wasm";

/// Copies assets from the client's project data dir into the dev assets.
fn copy_assets_to_project_dir() {
    let assets = assets_dir();

    for path in ["model/default.vrm", SCRIPT_PATH] {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../unavi-client/assets")
            .join(path);
        let dst = assets.join(path);
        if let Some(parent) = dst.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::copy(&src, &dst) {
            eprintln!("failed to copy vrm: {e}");
        }
    }
}

fn main() {
    copy_assets_to_project_dir();

    let (actor, blobs) = create_test_wds();

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(AssetPlugin {
                file_path: assets_dir().to_string_lossy().to_string(),
                ..Default::default()
            })
            .set(LogPlugin {
                filter: format!("{DEFAULT_FILTER},loro_internal=off"),
                ..Default::default()
            }),
        PanOrbitCameraPlugin,
        avian3d::PhysicsPlugins::default(),
        bevy_hsd::HsdPlugin,
        bevy_wds::WdsPlugin,
        unavi_avatar::AvatarPlugin,
        unavi_agent::AgentPlugin,
        unavi_script::ScriptPlugin,
    ))
    .add_observer(on_agent_load)
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
) {
    if *added {
        return;
    }
    *added = true;

    let mut cam = cameras.single_mut().expect("single camera");
    cam.is_active = false;

    commands.spawn((
        PanOrbitCamera::default(),
        Transform::from_xyz(-3.0, 5.0, -6.0).looking_at(Vec3::ZERO, Vec3::Y),
        RenderLayers::from_layers(&[0])
            .union(&DEFAULT_RENDER_LAYERS[&FirstPersonFlag::ThirdPersonOnly]),
    ));

    commands.trigger(SpawnLocalScript {
        permissions: ScriptPermissions::system(),
        source: ScriptSource::Path(SCRIPT_PATH.to_string()),
    });
}
