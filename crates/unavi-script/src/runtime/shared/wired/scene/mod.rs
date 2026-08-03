use std::{
    collections::HashSet,
    sync::Arc,
};

use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdNamespace,
};
use bevy_wds::{
    LocalBlobs,
    LocalDocs,
    blob::get::GetBlob,
};
use blake3::Hash;
use bytes::Bytes;
use hsd::{
    HSD_CONTAINER_ID,
    file::HsdFile,
};
use iroh_docs::NamespaceId;
use loro::{
    ExportMode,
    LoroDoc,
};
use unavi_quota::{
    Flow,
    Stock,
};
use unavi_space::anchor::ActiveSpace;
use unavi_util::{
    async_commands::AsyncCommands,
    async_task::spawn_async_task,
};

use crate::{
    error::ScriptError,
    firewall::{
        Access,
        Channel,
        Firewall,
    },
    quota::QuotaGuards,
    runtime::shared::{
        Api,
        registry::firewall::{
            FIREWALL_REGISTRY,
            validate_firewall,
        },
        slot_map::SlotMap,
        wired::scene::{
            document::DocRes,
            prim::PrimRes,
        },
    },
};

pub mod document;
pub mod prim;
pub mod util;

#[derive(Default)]
pub struct WiredSceneApi {
    pub docs:  SlotMap<DocRes>,
    pub prims: SlotMap<PrimRes>,
}

pub(super) async fn upload_blob(data: Vec<u8>) -> anyhow::Result<Hash> {
    let actor = bevy_wds::local_actor().ok_or_else(|| anyhow::anyhow!("no local actor"))?;
    actor.upload_blob(Bytes::from(data)).await
}

fn namespace(bytes: &[u8]) -> anyhow::Result<NamespaceId> {
    let arr: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("namespace id must be 32 bytes"))?;
    Ok(NamespaceId::from(&arr))
}

/// Exports `doc` to a Loro snapshot and writes it into a freshly created
/// namespace, returning the namespace id.
async fn create_snapshot_namespace(doc: &LoroDoc) -> anyhow::Result<NamespaceId> {
    doc.commit();
    let snapshot: Bytes = doc.export(ExportMode::Snapshot)?.into();
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let Some((docs, blobs)) = world
                .query::<(&LocalDocs, &LocalBlobs)>()
                .single(world)
                .ok()
                .map(|(d, b)| (d.0.clone(), b.0.clone()))
            else {
                return;
            };
            spawn_async_task(async move {
                let res = wds::space::create_snapshot_doc(&docs, &blobs, snapshot, &[]).await;
                tx.try_send(res).ok();
            });
        })
        .send()
        .await?;
    rx.recv().await?
}

/// Overwrites `ns`'s `snapshot` entry with `doc`'s current Loro state, so a
/// subsequently hosted namespace serves the script's live edits.
async fn write_namespace_snapshot(ns: NamespaceId, doc: &LoroDoc) -> anyhow::Result<()> {
    doc.commit();
    let snapshot: Bytes = doc.export(ExportMode::Snapshot)?.into();
    let (tx, rx) = async_channel::bounded(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let Some((docs, blobs)) = world
                .query::<(&LocalDocs, &LocalBlobs)>()
                .single(world)
                .ok()
                .map(|(d, b)| (d.0.clone(), b.0.clone()))
            else {
                return;
            };
            spawn_async_task(async move {
                let res = async {
                    let doc = docs
                        .api()
                        .open(ns)
                        .await?
                        .ok_or_else(|| anyhow::anyhow!("namespace {ns} not open"))?;
                    let author = docs.api().author_default().await?;
                    wds::space::write_snapshot(&doc, &blobs, author, snapshot, &[]).await
                }
                .await;
                tx.try_send(res).ok();
            });
        })
        .send()
        .await?;
    rx.recv().await?
}

async fn spawn_child_doc(api: &Api, doc: Arc<LoroDoc>, id: NamespaceId) -> Result<(), ScriptError> {
    let doc_guard = api.quota.charge(Stock::Documents, 1)?;

    let firewall = Firewall::for_child_doc(api.doc_id);
    FIREWALL_REGISTRY.write().insert(id, firewall.clone());
    // Seed the child's space now; the spawn command applies later.
    if let Some(parent_space) = unavi_space::membership::doc_space(api.doc_id) {
        unavi_space::membership::DOC_SPACE_REGISTRY
            .write()
            .insert(id, parent_space);
    }
    unavi_quota::registry::child_document_quota(id, api.doc_id);
    AsyncCommands::default()
        .spawn((
            Hsd(doc),
            HsdNamespace(id),
            firewall,
            api.permissions.clone(),
            QuotaGuards(vec![doc_guard]),
        ))
        .send()
        .await
        .map_err(|err| ScriptError::other(err.to_string()))?;
    Ok(())
}

pub async fn self_prim(api: &Api) -> anyhow::Result<u32> {
    let mut scene = api.wired_scene.lock().await;
    Ok(scene.prims.insert(
        PrimRes {
            doc:      Arc::clone(&api.doc),
            doc_id:   api.doc_id,
            id:       api.prim,
            is_proxy: false,
        },
        &api.quota,
    )?)
}

pub async fn self_document(api: &Api) -> anyhow::Result<u32> {
    let mut scene = api.wired_scene.lock().await;
    Ok(scene.docs.insert(
        DocRes {
            doc: Arc::clone(&api.doc),
            id:  api.doc_id,
        },
        &api.quota,
    )?)
}

pub async fn get_document(api: &Api, id: Vec<u8>) -> anyhow::Result<Option<u32>> {
    let id = namespace(&id)?;
    validate_firewall(&api.doc_id, &id, Channel::SceneRead)?;

    let mut scene = api.wired_scene.lock().await;

    let existing = scene.docs.iter().find(|(_, v)| v.id == id).map(|(k, _)| k);
    if let Some(key) = existing {
        return Ok(scene.docs.insert_clone(key, &api.quota).transpose()?);
    }

    if id == api.doc_id {
        return Ok(Some(scene.docs.insert(
            DocRes {
                doc: Arc::clone(&api.doc),
                id,
            },
            &api.quota,
        )?));
    }

    let (tx, rx) = async_channel::bounded::<Option<Arc<LoroDoc>>>(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let doc = world
                .query::<(&HsdNamespace, &Hsd)>()
                .iter(world)
                .find(|(rid, _)| rid.0 == id)
                .map(|(_, d)| Arc::clone(&d.0));
            tx.try_send(doc).ok();
        })
        .send()
        .await?;

    let Some(doc) = rx.recv().await? else {
        return Ok(None);
    };
    Ok(Some(scene.docs.insert(DocRes { doc, id }, &api.quota)?))
}

pub async fn remove_document(api: &Api, id: Vec<u8>) -> anyhow::Result<()> {
    let id = namespace(&id)?;
    if let Err(err) = validate_firewall(&api.doc_id, &id, Channel::SceneWrite) {
        debug!(?err, "remove_document denied by firewall, skipping");
        return Ok(());
    }

    let mut scene = api.wired_scene.lock().await;
    let key = scene
        .docs
        .iter()
        .find(|(_, v)| v.id == id)
        .map(|(k, _)| k)
        .ok_or_else(|| anyhow::anyhow!("resource not found"))?;
    let Some(doc) = scene.docs.remove(key) else {
        return Ok(());
    };
    drop(scene);

    AsyncCommands::default()
        .push(move |world: &mut World| {
            let mut query = world.query::<(Entity, &HsdNamespace)>();
            if let Some((entity, _)) = query.iter(world).find(|(_, v)| v.0 == doc.id) {
                world.despawn(entity);
            }
        })
        .send()
        .await?;

    Ok(())
}

pub async fn load_hsd(api: &Api, blob_id: Vec<u8>) -> Result<u32, ScriptError> {
    api.quota.spend(Flow::CreateDocument, 1.0)?;

    let (doc, id) = async {
        let blob_hash = Hash::from_slice(&blob_id)?;

        let bytes = {
            let (tx, rx) = async_channel::bounded(1);
            AsyncCommands::default()
                .trigger(GetBlob {
                    hash: blob_hash,
                    cancel: None,
                    tx,
                })
                .send()
                .await?;
            rx.recv().await?
        };

        let hsd_str = std::str::from_utf8(&bytes)?.to_owned();
        let file = HsdFile::from_ron(&hsd_str)?;

        let doc = LoroDoc::new();
        file.load_into_doc(&doc)?;
        let id = create_snapshot_namespace(&doc).await?;
        anyhow::Ok((Arc::new(doc), id))
    }
    .await?;

    spawn_child_doc(api, Arc::clone(&doc), id).await?;

    let mut scene = api.wired_scene.lock().await;
    Ok(scene.docs.insert(DocRes { doc, id }, &api.quota)?)
}

pub async fn publish_document(api: &Api, id: Vec<u8>) -> anyhow::Result<()> {
    let id = namespace(&id)?;
    validate_firewall(&api.doc_id, &id, Channel::SceneWrite)?;
    api.quota.spend(Flow::Publish, 1.0)?;

    let firewall = FIREWALL_REGISTRY.read().get(&id).cloned();
    if let Some(firewall) = firewall {
        firewall
            .0
            .write()
            .insert(Channel::SceneWrite, Access::Restricted(HashSet::new()));
    }

    let space = if let Some(s) = unavi_space::membership::doc_space(id) {
        s
    } else {
        // If document doesn't belong to a space, add it to the active space.
        let (tx, rx) = async_channel::bounded::<Option<NamespaceId>>(1);
        AsyncCommands::default()
            .push(move |world: &mut World| {
                let active = world.get_resource::<ActiveSpace>().and_then(|a| a.0);
                let ns = active.and_then(|e| world.get::<unavi_space::Space>(e).map(|s| s.0));
                tx.try_send(ns).ok();
            })
            .send()
            .await?;
        rx.recv()
            .await?
            .ok_or_else(|| anyhow::anyhow!("doc has no space and no active space"))?
    };

    // Checkpoint the script's live edits into the namespace snapshot so hosts
    // and peers serve the current state, then ask the home server to host it.
    let doc = {
        let scene = api.wired_scene.lock().await;
        scene
            .docs
            .iter()
            .find_map(|(_, d)| (d.id == id).then(|| Arc::clone(&d.doc)))
    };
    let Some(doc) = doc else {
        anyhow::bail!("published doc not held by script");
    };
    write_namespace_snapshot(id, &doc).await?;

    let actor = bevy_wds::local_actor().ok_or_else(|| anyhow::anyhow!("no local actor"))?;
    actor.host_doc(id).await?;

    // Pin the document locally; ownership follows from the pin, and the pin's
    // quota is charged to the resulting owner.
    if !unavi_space::state::entities::self_pin(space, id).await {
        anyhow::bail!("space state not tracked locally or pin over quota");
    }

    Ok(())
}

pub async fn create_document(api: &Api) -> Result<u32, ScriptError> {
    api.quota.spend(Flow::CreateDocument, 1.0)?;

    let (doc, id) = async {
        let doc = LoroDoc::new();
        // Ensure the HSD tree container exists.
        let _ = doc.get_tree(&*HSD_CONTAINER_ID);
        let id = create_snapshot_namespace(&doc).await?;
        anyhow::Ok((Arc::new(doc), id))
    }
    .await?;

    spawn_child_doc(api, Arc::clone(&doc), id).await?;

    let mut scene = api.wired_scene.lock().await;
    Ok(scene.docs.insert(DocRes { doc, id }, &api.quota)?)
}
