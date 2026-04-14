use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use bevy_wds::LocalActor;
use bytes::Bytes;
use loro::{LoroDoc, LoroList, LoroMap, LoroTree, TreeParentId};

use crate::{HsdDoc, HsdRecordId};

/// Triggered on an entity to queue an HSD file load for it.
#[derive(EntityEvent, Clone)]
pub struct LoadHsdFile {
    pub entity: Entity,
    pub path: PathBuf,
}

/// Stamped on entities waiting for a WDS actor before loading can begin.
#[derive(Component)]
pub struct HsdFilePath(pub PathBuf);

struct PendingLoad {
    entity: Entity,
    rx: std::sync::mpsc::Receiver<anyhow::Result<Arc<LoroDoc>>>,
}

#[derive(Resource)]
pub struct PendingHsdLoads(Mutex<Vec<PendingLoad>>);

impl Default for PendingHsdLoads {
    fn default() -> Self {
        Self(Mutex::new(Vec::new()))
    }
}

/// Observer: store the path on the entity so `start_hsd_loads` can pick it up
/// once a WDS actor is available.
pub(crate) fn on_load_hsd_file(trigger: On<LoadHsdFile>, mut commands: Commands) {
    let event = trigger.event();
    let name = event
        .path
        .file_stem()
        .map_or_else(|| "hsd".to_string(), |s| s.to_string_lossy().into_owned());
    commands
        .entity(event.entity)
        .insert((HsdFilePath(event.path.clone()), Name::new(name)));
}

/// System: once a `LocalActor` exists, start async loads for all queued entities.
pub fn start_hsd_loads(
    queued: Query<(Entity, &HsdFilePath)>,
    actor_query: Query<&LocalActor>,
    pending: Res<PendingHsdLoads>,
    mut commands: Commands,
) {
    let Ok(local_actor) = actor_query.single() else {
        return;
    };
    let actor = local_actor.0.clone();

    for (entity, HsdFilePath(path)) in &queued {
        let path = path.clone();
        let actor = actor.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        pending
            .0
            .lock()
            .expect("pending lock")
            .push(PendingLoad { entity, rx });
        unavi_wasm_compat::spawn_thread(async move {
            let _ = tx.send(build_hsd_doc_from_file(path, actor).await);
        });
        commands.entity(entity).remove::<HsdFilePath>();
    }
}

pub fn poll_hsd_file_loads(pending: Res<PendingHsdLoads>, mut commands: Commands) {
    let mut loads = pending.0.lock().expect("pending lock");
    let mut still_pending = Vec::new();
    for p in loads.drain(..) {
        match p.rx.try_recv() {
            Ok(Ok(doc)) => {
                let doc_id = doc
                    .export(loro::ExportMode::Snapshot)
                    .map_or_else(|_| blake3::hash(&[]), |b| blake3::hash(&b));
                commands
                    .entity(p.entity)
                    .insert((HsdDoc(doc), HsdRecordId(doc_id)));
            }
            Ok(Err(err)) => {
                error!("LoadHsdFile failed: {err:?}");
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                still_pending.push(p);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {}
        }
    }
    *loads = still_pending;
}

fn build_hsd_doc_from_file(
    path: PathBuf,
    actor: wds::actor::Actor,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Arc<LoroDoc>>> + Send>> {
    Box::pin(async move {
        use anyhow::Context;
        let src = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let hsd_file =
            hsd::HsdFile::parse(&src).with_context(|| format!("parsing {}", path.display()))?;
        let base_dir = path
            .parent()
            .unwrap_or_else(|| std::path::Path::new(""))
            .to_path_buf();
        build_hsd_doc(hsd_file, base_dir, actor).await
    })
}

fn build_hsd_doc(
    hsd_file: hsd::HsdFile,
    base_dir: PathBuf,
    actor: wds::actor::Actor,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Arc<LoroDoc>>> + Send>> {
    Box::pin(async move {
        use anyhow::Context;
        let doc = Arc::new(LoroDoc::new());
        let hsd_map = doc.get_map("hsd");

        let assets_container = hsd_map
            .get_or_create_container("assets", LoroMap::new())
            .context("creating assets container")?;

        #[expect(clippy::case_sensitive_file_extension_comparisons)]
        for (name, asset_path) in &hsd_file.assets {
            if asset_path.ends_with(".hsd") {
                let full_path = base_dir.join(asset_path);
                let sub_doc = build_hsd_doc_from_file(full_path, actor.clone()).await?;
                let snapshot = sub_doc
                    .export(loro::ExportMode::Snapshot)
                    .map_err(|e| anyhow::anyhow!("export sub-doc snapshot: {e:?}"))?;
                let hash = actor
                    .upload_blob(Bytes::from(snapshot))
                    .await
                    .map_err(|e| anyhow::anyhow!("upload sub-hsd: {e}"))?;
                assets_container
                    .insert(
                        name.as_str(),
                        loro::LoroValue::Binary(hash.as_bytes().to_vec().into()),
                    )
                    .context("inserting asset hash")?;
            }
        }

        let nodes_tree = hsd_map
            .get_or_create_container("nodes", LoroTree::new())
            .context("creating nodes tree")?;

        for (node_name, node_def) in &hsd_file.nodes {
            let node_id = nodes_tree
                .create(TreeParentId::Root)
                .context("creating tree node")?;
            let meta = nodes_tree.get_meta(node_id).context("getting node meta")?;

            meta.insert("name", node_name.as_str())
                .context("setting node name")?;

            if !node_def.scripts.is_empty() {
                let scripts_list = meta
                    .get_or_create_container("scripts", LoroList::new())
                    .context("creating scripts list")?;

                for script_path in &node_def.scripts {
                    let full_path = base_dir.join(script_path);
                    let bytes = std::fs::read(&full_path)
                        .with_context(|| format!("reading script {}", full_path.display()))?;
                    let hash = actor
                        .upload_blob(Bytes::from(bytes))
                        .await
                        .map_err(|e| anyhow::anyhow!("upload script: {e}"))?;
                    scripts_list
                        .push(hash.as_bytes().to_vec())
                        .context("pushing script hash")?;
                }
            }
        }

        Ok(doc)
    })
}

/// Read the `assets` map from a hydrated HSD document.
#[must_use]
pub fn read_hsd_assets(
    hsd_map: &loro::LoroMap,
) -> std::collections::BTreeMap<smol_str::SmolStr, blake3::Hash> {
    use loro::LoroValue;

    let value = hsd_map.get_deep_value();
    let LoroValue::Map(root) = &value else {
        return std::collections::BTreeMap::default();
    };
    let Some(LoroValue::Map(assets)) = root.get("assets") else {
        return std::collections::BTreeMap::default();
    };
    assets
        .iter()
        .filter_map(|(k, v)| {
            if let LoroValue::Binary(bytes) = v {
                let arr: [u8; 32] = bytes[..].try_into().ok()?;
                Some((smol_str::SmolStr::new(k), blake3::Hash::from_bytes(arr)))
            } else {
                None
            }
        })
        .collect()
}
