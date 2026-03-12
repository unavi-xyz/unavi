use std::sync::{Arc, mpsc};

use bevy::prelude::*;
use bevy_hsd::{HsdDoc, HsdRecordId};
use bevy_wds::LocalActor;
use bytes::Bytes;
use loro::{LoroDoc, LoroList, LoroTree, TreeParentId};

use crate::{asset::Wasm, permissions::ScriptPermissions};

#[derive(Clone)]
pub enum ScriptSource {
    Bytes(Vec<u8>),
    Path(String),
}

#[derive(Event, Clone)]
pub struct SpawnLocalScript {
    pub permissions: ScriptPermissions,
    pub source: ScriptSource,
}

struct PendingHandle {
    handle: Handle<Wasm>,
    name: Option<String>,
    permissions: ScriptPermissions,
}

pub struct PendingUpload {
    actor: wds::actor::Actor,
    name: Option<String>,
    permissions: ScriptPermissions,
    rx: mpsc::Receiver<anyhow::Result<blake3::Hash>>,
}

pub struct RecordPending {
    name: Option<String>,
    permissions: ScriptPermissions,
    rx: mpsc::Receiver<anyhow::Result<(blake3::Hash, LoroDoc)>>,
}

#[derive(Resource, Default)]
pub struct PendingHandles(Vec<PendingHandle>);

pub fn on_spawn_local_script(
    trigger: On<SpawnLocalScript>,
    server: Res<AssetServer>,
    mut assets: ResMut<Assets<Wasm>>,
    mut pending: ResMut<PendingHandles>,
) {
    let ev = trigger.event();
    let (handle, name) = match &ev.source {
        ScriptSource::Path(path) => {
            let name = path_to_name(path);
            (server.load::<Wasm>(path), Some(name))
        }
        ScriptSource::Bytes(bytes) => (assets.add(Wasm(bytes.clone())), None),
    };
    pending.0.push(PendingHandle {
        handle,
        name,
        permissions: ev.permissions.clone(),
    });
}

pub fn poll_local_scripts(
    mut commands: Commands,
    wasm_assets: Res<Assets<Wasm>>,
    actors: Query<&LocalActor>,
    mut handles: ResMut<PendingHandles>,
    mut uploads: Local<Vec<PendingUpload>>,
    mut records: Local<Vec<RecordPending>>,
) {
    #[cfg(target_family = "wasm")]
    return;

    // Poll completed record creations → spawn entities
    let mut still_creating = Vec::new();
    for pending in records.drain(..) {
        match pending.rx.try_recv() {
            Ok(Ok((id, doc))) => {
                spawn_local_hsd_doc(
                    &mut commands,
                    id,
                    Arc::new(doc),
                    pending.permissions,
                    pending.name,
                );
            }
            Ok(Err(e)) => {
                error!("local script record create: {e}");
            }
            Err(mpsc::TryRecvError::Empty) => still_creating.push(pending),
            Err(mpsc::TryRecvError::Disconnected) => {}
        }
    }
    *records = still_creating;

    // Poll completed uploads → start record creation
    let mut still_uploading = Vec::new();
    for pending in uploads.drain(..) {
        match pending.rx.try_recv() {
            Ok(Ok(hash)) => {
                let actor = pending.actor;
                let (tx, rx) = mpsc::channel();
                unavi_wasm_compat::spawn_thread(async move {
                    let _ = tx.send(create_hsd_record(actor, hash).await);
                });
                records.push(RecordPending {
                    name: pending.name,
                    permissions: pending.permissions,
                    rx,
                });
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
            let upload_actor = actor.0.clone();
            let record_actor = actor.0.clone();
            let (tx, rx) = mpsc::channel();
            unavi_wasm_compat::spawn_thread(async move {
                let _ = tx.send(upload_actor.upload_blob(bytes).await);
            });
            uploads.push(PendingUpload {
                actor: record_actor,
                name: pending.name,
                permissions: pending.permissions,
                rx,
            });
        } else {
            still_pending.push(pending);
        }
    }
    handles.0 = still_pending;
}

fn path_to_name(path: &str) -> String {
    path.strip_prefix("wasm/")
        .unwrap_or(path)
        .strip_suffix(".wasm")
        .unwrap_or(path)
        .replace('/', ":")
}

async fn create_hsd_record(
    actor: wds::actor::Actor,
    hash: blake3::Hash,
) -> anyhow::Result<(blake3::Hash, LoroDoc)> {
    let result = actor
        .create_record()
        .add_schema("hsd", &*wired_schemas::schemas::SCHEMA_HSD, |doc| {
            let hsd = doc.get_map("hsd");
            let nodes = hsd
                .get_or_create_container("nodes", LoroTree::new())
                .map_err(|e| anyhow::anyhow!("nodes tree: {e}"))?;
            let node_id = nodes
                .create(TreeParentId::Root)
                .map_err(|e| anyhow::anyhow!("create node: {e}"))?;
            let meta = nodes
                .get_meta(node_id)
                .map_err(|e| anyhow::anyhow!("node meta: {e}"))?;
            let scripts = meta
                .get_or_create_container("scripts", LoroList::new())
                .map_err(|e| anyhow::anyhow!("scripts list: {e}"))?;
            scripts
                .push(hash.as_bytes().to_vec())
                .map_err(|e| anyhow::anyhow!("push hash: {e}"))?;
            doc.commit();
            Ok(())
        })?
        .send()
        .await?;
    Ok((result.id, result.doc))
}

fn spawn_local_hsd_doc(
    commands: &mut Commands,
    id: blake3::Hash,
    doc: Arc<LoroDoc>,
    perms: ScriptPermissions,
    name: Option<String>,
) {
    let mut entity = commands.spawn((HsdDoc(doc), HsdRecordId(id), perms));
    if let Some(name) = name {
        entity.insert(Name::new(name));
    }
}
