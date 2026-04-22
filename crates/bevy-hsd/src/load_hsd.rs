use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;
use bevy_wds::LocalActor;
use bytes::Bytes;
use loro::{LoroDoc, LoroList, LoroMap, LoroTree, LoroValue, TreeParentId};
use unavi_util::async_task::spawn_async_task;

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
    let Ok(actor) = actor_query.single().map(|a| a.0.clone()) else {
        return;
    };

    for (entity, HsdFilePath(path)) in &queued {
        let path = path.clone();
        let actor = actor.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        pending
            .0
            .lock()
            .expect("pending lock")
            .push(PendingLoad { entity, rx });

        spawn_async_task(async move {
            let _ = tx.send(build_hsd_doc_from_file(path, &[actor]).await);
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
                error!(?err, "LoadHsdFile failed");
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                still_pending.push(p);
            }
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {}
        }
    }
    *loads = still_pending;
}

#[must_use]
pub fn build_hsd_doc_from_file(
    path: PathBuf,
    actors: &[wds::actor::Actor],
) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Arc<LoroDoc>>> + Send>> {
    let actors = actors.to_vec();
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
        build_hsd_doc(hsd_file, base_dir, &actors).await
    })
}

#[expect(clippy::too_many_lines)]
fn build_hsd_doc(
    hsd_file: hsd::HsdFile,
    base_dir: PathBuf,
    actors: &[wds::actor::Actor],
) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<Arc<LoroDoc>>> + Send>> {
    let actors = actors.to_vec();
    Box::pin(async move {
        use anyhow::Context;
        let doc = Arc::new(LoroDoc::new());
        let hsd_map = doc.get_map("hsd");

        // Assets
        let assets_container = hsd_map
            .get_or_create_container("assets", LoroMap::new())
            .context("creating assets container")?;

        #[expect(clippy::case_sensitive_file_extension_comparisons)]
        for (name, asset_path) in &hsd_file.assets {
            if asset_path.ends_with(".hsd") {
                let full_path = base_dir.join(asset_path);
                let sub_doc = build_hsd_doc_from_file(full_path, &actors).await?;
                let snapshot = sub_doc
                    .export(loro::ExportMode::Snapshot)
                    .map_err(|e| anyhow::anyhow!("export sub-doc snapshot: {e:?}"))?;
                let hash = upload_blob(Bytes::from(snapshot), &actors).await?;
                assets_container
                    .insert(
                        name.as_str(),
                        LoroValue::Binary(hash.as_bytes().to_vec().into()),
                    )
                    .context("inserting asset hash")?;
            }
        }

        // Images
        if !hsd_file.images.is_empty() {
            let images_container = hsd_map
                .get_or_create_container("images", LoroMap::new())
                .context("creating images container")?;

            for (key, img) in &hsd_file.images {
                let full_path = base_dir.join(&img.path);
                let bytes = std::fs::read(&full_path)
                    .with_context(|| format!("reading image {}", full_path.display()))?;
                let hash = upload_blob(Bytes::from(bytes), &actors).await?;

                let img_map = images_container
                    .get_or_create_container(key.as_str(), LoroMap::new())
                    .context("creating image map")?;

                img_map
                    .insert("data", LoroValue::Binary(hash.as_bytes().to_vec().into()))
                    .context("inserting image data hash")?;

                if let Some(v) = img.address_mode_u {
                    img_map
                        .insert("address_mode_u", v)
                        .context("address_mode_u")?;
                }
                if let Some(v) = img.address_mode_v {
                    img_map
                        .insert("address_mode_v", v)
                        .context("address_mode_v")?;
                }
                if let Some(v) = img.address_mode_w {
                    img_map
                        .insert("address_mode_w", v)
                        .context("address_mode_w")?;
                }
                if let Some(v) = img.mag_filter {
                    img_map.insert("mag_filter", v).context("mag_filter")?;
                }
                if let Some(v) = img.min_filter {
                    img_map.insert("min_filter", v).context("min_filter")?;
                }
                if let Some(v) = img.mipmap_filter {
                    img_map
                        .insert("mipmap_filter", v)
                        .context("mipmap_filter")?;
                }
                if let Some(ref v) = img.name {
                    img_map.insert("name", v.as_str()).context("image name")?;
                }
                if let Some(v) = img.srgb {
                    img_map.insert("srgb", v).context("srgb")?;
                }
            }
        }

        // Materials
        if !hsd_file.materials.is_empty() {
            let materials_container = hsd_map
                .get_or_create_container("materials", LoroMap::new())
                .context("creating materials container")?;

            for (key, mat) in &hsd_file.materials {
                let mat_map = materials_container
                    .get_or_create_container(key.as_str(), LoroMap::new())
                    .context("creating material map")?;

                if let Some(ref v) = mat.name {
                    mat_map
                        .insert("name", v.as_str())
                        .context("material name")?;
                }
                if let Some(ref c) = mat.base_color {
                    let l = LoroList::new();
                    for &v in c {
                        l.push(v).context("base_color push")?;
                    }
                    mat_map
                        .insert_container("base_color", l)
                        .context("base_color")?;
                }
                if let Some(ref v) = mat.base_color_texture {
                    mat_map
                        .insert("base_color_texture", v.as_str())
                        .context("base_color_texture")?;
                }
                if let Some(v) = mat.roughness {
                    mat_map.insert("roughness", v).context("roughness")?;
                }
                if let Some(v) = mat.metallic {
                    mat_map.insert("metallic", v).context("metallic")?;
                }
                if let Some(v) = mat.alpha_cutoff {
                    mat_map.insert("alpha_cutoff", v).context("alpha_cutoff")?;
                }
                if let Some(ref v) = mat.alpha_mode {
                    mat_map
                        .insert("alpha_mode", v.as_str())
                        .context("alpha_mode")?;
                }
                if let Some(v) = mat.double_sided {
                    mat_map.insert("double_sided", v).context("double_sided")?;
                }
                if let Some(ref c) = mat.emissive {
                    let l = LoroList::new();
                    for &v in c {
                        l.push(v).context("emissive push")?;
                    }
                    mat_map
                        .insert_container("emissive", l)
                        .context("emissive")?;
                }
                if let Some(ref v) = mat.emissive_texture {
                    mat_map
                        .insert("emissive_texture", v.as_str())
                        .context("emissive_texture")?;
                }
                if let Some(ref v) = mat.metallic_roughness_texture {
                    mat_map
                        .insert("metallic_roughness_texture", v.as_str())
                        .context("metallic_roughness_texture")?;
                }
                if let Some(ref v) = mat.normal_texture {
                    mat_map
                        .insert("normal_texture", v.as_str())
                        .context("normal_texture")?;
                }
                if let Some(ref v) = mat.occlusion_texture {
                    mat_map
                        .insert("occlusion_texture", v.as_str())
                        .context("occlusion_texture")?;
                }
            }
        }

        // Nodes
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
                    let hash = upload_blob(Bytes::from(bytes), &actors).await?;
                    scripts_list
                        .push(hash.as_bytes().to_vec())
                        .context("pushing script hash")?;
                }
            }
        }

        Ok(doc)
    })
}

async fn upload_blob(bytes: Bytes, actors: &[wds::actor::Actor]) -> anyhow::Result<blake3::Hash> {
    let mut hash = None;
    for actor in actors {
        let h = actor
            .upload_blob(bytes.clone())
            .await
            .map_err(|e| anyhow::anyhow!("upload blob: {e}"))?;
        hash = Some(h);
    }
    hash.ok_or_else(|| anyhow::anyhow!("no actors to upload blob to"))
}

/// Read the `assets` map from a hydrated HSD document.
#[must_use]
pub fn read_hsd_assets(
    hsd_map: &loro::LoroMap,
) -> std::collections::BTreeMap<smol_str::SmolStr, blake3::Hash> {
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
