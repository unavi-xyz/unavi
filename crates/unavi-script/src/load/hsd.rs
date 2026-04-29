use bevy::prelude::*;
use bevy_hsd::HsdScript;
use bevy_wds::LocalBlobs;

pub fn spawn_hsd_scripts(
    trigger: On<Add, HsdScript>,
    scripts: Query<&HsdScript>,
    blobs: Query<&LocalBlobs>,
) {
    let Ok(blobs) = blobs.single() else {
        warn!("Can't load script, no LocalBlobs");
        return;
    };

    let script = scripts.get(trigger.entity).expect("get scripts");

    // Load bin from WDS.

    info!("spawning script {}", script.0);
}

pub fn despawn_hsd_scripts(trigger: On<Remove, HsdScript>, scripts: Query<&HsdScript>) {}
