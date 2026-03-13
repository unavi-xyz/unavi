use std::{path::PathBuf, sync::LazyLock};

use avian3d::prelude::Gravity;
use bevy::{
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
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

fn main() {
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
        unavi_input::InputPlugin,
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

fn init_scene(mut commands: Commands, mut gravity: ResMut<Gravity>) {
    gravity.0 = Vec3::ZERO;

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(5.0, 8.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
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
    ));

    commands.trigger(SpawnLocalScript {
        permissions: ScriptPermissions::system(),
        source: ScriptSource::Path("wasm/example/wired_agent.wasm".to_string()),
    });
}
