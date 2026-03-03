use std::sync::{Arc, mpsc};

use bevy::prelude::*;
use bevy_hsd::HsdDoc;
use bevy_wds::LocalActor;
use bytes::Bytes;
use loro::{LoroDoc, LoroList, LoroTree, TreeParentId};

use crate::{asset::Wasm, permissions::ScriptPermissions};

#[derive(Event, Clone)]
pub struct SpawnLocalScript {
    pub path: String,
    pub permissions: ScriptPermissions,
}

struct PendingHandle {
    handle: Handle<Wasm>,
    permissions: ScriptPermissions,
}

pub struct PendingUpload {
    rx: mpsc::Receiver<anyhow::Result<blake3::Hash>>,
    permissions: ScriptPermissions,
}

#[derive(Resource, Default)]
pub struct PendingHandles(Vec<PendingHandle>);

pub fn on_spawn_local_script(
    trigger: On<SpawnLocalScript>,
    server: Res<AssetServer>,
    mut pending: ResMut<PendingHandles>,
) {
    let ev = trigger.event();
    let handle = server.load::<Wasm>(&ev.path);
    pending.0.push(PendingHandle {
        handle,
        permissions: ev.permissions.clone(),
    });
}

pub fn poll_local_scripts(
    mut commands: Commands,
    wasm_assets: Res<Assets<Wasm>>,
    actors: Query<&LocalActor>,
    mut handles: ResMut<PendingHandles>,
    mut uploads: Local<Vec<PendingUpload>>,
) {
    #[cfg(target_family = "wasm")]
    return;

    // Poll completed uploads
    let mut still_uploading = Vec::new();
    for pending in uploads.drain(..) {
        match pending.rx.try_recv() {
            Ok(Ok(hash)) => {
                spawn_local_hsd_doc(&mut commands, hash, pending.permissions);
            }
            Ok(Err(e)) => {
                error!("local script upload: {e}");
            }
            Err(mpsc::TryRecvError::Empty) => still_uploading.push(pending),
            Err(mpsc::TryRecvError::Disconnected) => {}
        }
    }
    *uploads = still_uploading;

    // Check if assets are ready and start uploads
    let Ok(actor) = actors.single() else { return };

    let mut still_pending = Vec::new();
    for pending in handles.0.drain(..) {
        if let Some(wasm) = wasm_assets.get(&pending.handle) {
            let bytes = Bytes::from(wasm.0.clone());
            let actor = actor.0.clone();
            let (tx, rx) = mpsc::channel();
            let permissions = pending.permissions;
            unavi_wasm_compat::spawn_thread(async move {
                let _ = tx.send(actor.upload_blob(bytes).await);
            });
            uploads.push(PendingUpload { rx, permissions });
        } else {
            still_pending.push(pending);
        }
    }
    handles.0 = still_pending;
}

fn spawn_local_hsd_doc(commands: &mut Commands, hash: blake3::Hash, perms: ScriptPermissions) {
    let doc = Arc::new(LoroDoc::new());
    {
        let hsd = doc.get_map("hsd");
        let nodes = hsd
            .get_or_create_container("nodes", LoroTree::new())
            .expect("nodes tree");
        let node_id = nodes.create(TreeParentId::Root).expect("create node");
        let meta = nodes.get_meta(node_id).expect("node meta");
        let scripts = meta
            .get_or_create_container("scripts", LoroList::new())
            .expect("scripts list");
        scripts.push(hash.as_bytes().to_vec()).expect("push hash");
        doc.commit();
    }
    commands.spawn((HsdDoc(doc), perms));
}
