use std::{collections::HashMap, sync::Arc};

use bevy::prelude::*;
use bevy_hsd::HsdPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_vrm::BoneName;
use bevy_wds::{LocalActor, LocalBlobs, WdsPlugin, util::create_test_wds};
use loro::{LoroDoc, LoroTree, TreeParentId};
use unavi_script::{
    ScriptPermissions, ScriptPlugin, SpawnLocalScript, agent::LocalAgentHsdDoc,
    load::local::ScriptSource,
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
    ))
    .add_systems(Startup, init_scene);

    app.world_mut()
        .spawn((LocalActor(actor), LocalBlobs(blobs)));

    // Build a minimal agent HSD doc with proxy nodes for the bones the
    // example script uses.
    app.world_mut().insert_resource(make_agent_hsd_doc(&[
        BoneName::RightHand,
        BoneName::RightUpperArm,
    ]));

    app.world_mut().trigger(SpawnLocalScript {
        permissions: ScriptPermissions {
            wired_agent: true,
            wired_local_agent: true,
            ..Default::default()
        },
        source: ScriptSource::Path("wasm/example/agent.wasm".to_string()),
    });

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
}

fn make_agent_hsd_doc(bones: &[BoneName]) -> LocalAgentHsdDoc {
    let doc = Arc::new(LoroDoc::new());
    let mut bone_nodes = HashMap::new();

    let tree: LoroTree = doc
        .get_map("hsd")
        .get_or_create_container("nodes", LoroTree::new())
        .expect("nodes tree");

    for &bone in bones {
        let node_id = tree.create(TreeParentId::Root).expect("create proxy node");
        let meta = tree.get_meta(node_id).expect("meta");
        let bone_str = format!("{bone}");
        meta.insert("bone_name", bone_str.trim_matches('"'))
            .expect("insert bone_name");
        bone_nodes.insert(bone, node_id);
    }

    doc.commit();

    LocalAgentHsdDoc {
        doc,
        bone_nodes: Arc::new(bone_nodes),
    }
}
