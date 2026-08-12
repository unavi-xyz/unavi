//! The VUI gallery.
//!
//! Walk up to the orbit with the real player controller, aim with the mouse,
//! and click to open a mote or drag one out.

use std::{
    path::PathBuf,
    sync::LazyLock,
};

use avian3d::prelude::*;
use bevy::{
    log::{
        DEFAULT_FILTER,
        LogPlugin,
    },
    prelude::*,
};
use bevy_hsd::load::LoadHsd;
use bevy_wds::{
    LocalActor,
    LocalBlobStore,
    LocalBlobs,
    LocalDocs,
};
use directories::ProjectDirs;
use unavi_agent::LocalAgent;
use unavi_script::permissions::ApiPermissions;

use crate::util::create_test_wds;

mod util;

const SCRIPT_PATH: &str = "hsd/example_unavi_vui.hsdz";
const ASSETS_PATH: &str = "../unavi-client/assets/";
const GROUND_SIZE: f32 = 24.0;
const GROUND_THICKNESS: f32 = 0.5;

pub static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("", "UNAVI", "unavi-client").expect("project dirs");
    std::fs::create_dir_all(dirs.data_local_dir()).expect("data local dir");
    dirs
});

/// The agent controller loads its VRM and animations from the client's asset
/// directory, which only the client itself populates.
fn copy_runtime_assets() -> anyhow::Result<()> {
    let src = DIRS.data_local_dir().join("assets");
    let dst = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(ASSETS_PATH)
        .canonicalize()?;
    std::fs::create_dir_all(dst.join("model"))?;
    for path in ["model/animations.glb", "model/default.vrm"] {
        std::fs::copy(src.join(path), dst.join(path))?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    copy_runtime_assets()?;

    let wds = create_test_wds();

    let mut app = App::new();
    // Registers the `iroh://` asset source, which must exist before
    // `AssetPlugin` builds the sources it knows about.
    app.add_plugins(unavi_assets_fetch::UnaviAssetsFetchPlugin);
    app.add_plugins((
        DefaultPlugins
            .set(AssetPlugin {
                file_path: ASSETS_PATH.to_string(),
                ..Default::default()
            })
            .set(LogPlugin {
                filter: DEFAULT_FILTER.to_string(),
                ..Default::default()
            }),
        unavi_physics::PhysicsPlugin,
        bevy_hsd::HsdPlugin,
        bevy_iroh::IrohPlugin,
        bevy_wds::WdsPlugin,
        unavi_util::UtilPlugin,
        unavi_input::InputPlugin,
        unavi_grab::GrabPlugin,
        unavi_agent::AgentPlugin,
        unavi_avatar::AvatarPlugin,
        unavi_script::ScriptPlugin,
    ))
    .insert_resource(ClearColor(Color::srgb(0.05, 0.06, 0.09)))
    .add_systems(Startup, init_scene);

    app.world_mut()
        .spawn((
            LocalActor(wds.actor),
            LocalBlobStore(wds.store),
            LocalBlobs(wds.blobs),
            LocalDocs(wds.docs),
        ));

    app.run();

    Ok(())
}

fn init_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        DirectionalLight {
            illuminance: 4_000.0,
            shadow_maps_enabled: true,
            ..Default::default()
        },
        Transform::from_xyz(5.0, 8.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Name::new("ground"),
        Mesh3d(meshes.add(Cuboid::new(GROUND_SIZE, GROUND_THICKNESS, GROUND_SIZE))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.17, 0.20),
            perceptual_roughness: 0.9,
            ..Default::default()
        })),
        Transform::from_xyz(0.0, -GROUND_THICKNESS / 2.0, 0.0),
        RigidBody::Static,
        Collider::cuboid(GROUND_SIZE, GROUND_THICKNESS, GROUND_SIZE),
    ));

    commands.spawn(LocalAgent);

    // The script retries `local-camera` until the agent's proxy exists, so it
    // does not have to wait for the avatar to finish loading.
    let handle = asset_server.load(SCRIPT_PATH);
    commands.spawn((
        LoadHsd {
            handle,
            on_load: None,
        },
        // Scene writes are refused for a document in no space; a standalone
        // harness needs the shell's own permissions.
        ApiPermissions::system(),
    ));
}
