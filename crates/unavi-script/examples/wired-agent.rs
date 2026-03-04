use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use directories::ProjectDirs;

use bevy::{
    log::{DEFAULT_FILTER, LogPlugin},
    prelude::*,
};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_vrm::BoneName;
use bevy_wds::{LocalActor, LocalBlobs, WdsPlugin, util::create_test_wds};
use unavi_avatar::{Avatar, AvatarBones, AvatarBonesPopulated, AvatarPlugin};
use unavi_script::{
    ScriptPermissions, ScriptPlugin, SpawnLocalScript, agent::LocalAgentDocs,
    load::local::ScriptSource,
};

/// Copies the VRM from the client's project data dir into the dev assets
/// dir if it exists there. The client downloads the VRM on first run;
/// this lets the example use it without duplicating download logic.
fn copy_vrm_from_project_dir() {
    const VRM_PATH: &str = "model/default.vrm";

    let Some(proj) = ProjectDirs::from("", "UNAVI", "unavi-client") else {
        return;
    };
    let src = proj.data_local_dir().join("assets").join(VRM_PATH);
    if !src.exists() {
        return;
    }

    let dst = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../unavi-client/assets")
        .join(VRM_PATH);
    if let Some(parent) = dst.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Err(e) = std::fs::copy(&src, &dst) {
        eprintln!("failed to copy vrm: {e}");
    }
}

fn main() {
    copy_vrm_from_project_dir();

    let (actor, blobs) = create_test_wds();

    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(AssetPlugin {
                file_path: "../unavi-client/assets".to_string(),
                ..Default::default()
            })
            .set(LogPlugin {
                filter: format!("{},loro_internal=off", DEFAULT_FILTER),
                ..Default::default()
            }),
        AvatarPlugin,
        PanOrbitCameraPlugin,
        WdsPlugin,
        bevy_hsd::HsdPlugin,
        ScriptPlugin,
    ))
    .add_systems(Startup, init_scene)
    .add_systems(Update, init_agent_docs);

    app.world_mut()
        .spawn((LocalActor(actor), LocalBlobs(blobs)));

    app.run();
}

fn init_scene(mut commands: Commands) {
    commands.spawn(Avatar);

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(5.0, 8.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        PanOrbitCamera::default(),
        Transform::from_xyz(-3.0, 5.0, -6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Waits for avatar bones to populate, then creates `LocalAgentDocs` and
/// triggers script load.
fn init_agent_docs(
    mut commands: Commands,
    avatars: Query<&AvatarBones, Added<AvatarBonesPopulated>>,
    existing: Option<Res<LocalAgentDocs>>,
) {
    if existing.is_some() {
        return;
    }
    let Ok(bones) = avatars.single() else {
        return;
    };

    let bone_entities: HashMap<BoneName, Entity> = bones.iter().map(|(&b, &e)| (b, e)).collect();
    commands.insert_resource(LocalAgentDocs {
        bone_entities: Arc::new(bone_entities),
        docs: Arc::new(Mutex::new(vec![])),
    });

    commands.trigger(SpawnLocalScript {
        permissions: ScriptPermissions {
            wired_agent: true,
            wired_local_agent: true,
            ..Default::default()
        },
        source: ScriptSource::Path("wasm/example/agent.wasm".to_string()),
    });
}
