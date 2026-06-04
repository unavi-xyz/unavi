use std::{
    collections::HashSet,
    sync::Arc,
};

use bevy::prelude::*;
use bevy_hsd::{
    Hsd,
    HsdRecordId,
};
use bevy_wds::{
    LocalActor,
    blob::get::GetBlob,
    record::{
        read::ReadRecord,
        write::{
            SchemaDef,
            WriteRecord,
        },
    },
};
use blake3::Hash;
use bytes::Bytes;
use hsd::{
    HSD_CONTAINER_ID,
    file::HsdFile,
};
use loro::LoroDoc;
use unavi_util::{
    async_commands::AsyncCommands,
    async_task::spawn_async_task,
};
use wired_schemas::SCHEMA_HSD;

use crate::{
    firewall::{
        Access,
        Channel,
        Firewall,
    },
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
    let (tx, rx) = async_channel::bounded::<anyhow::Result<Hash>>(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let actor = world
                .query::<&LocalActor>()
                .single(world)
                .map(|a| a.0.clone());
            match actor {
                Err(_) => {
                    tx.try_send(Err(anyhow::anyhow!("no local actor"))).ok();
                }
                Ok(actor) => {
                    let bytes = Bytes::from(data);
                    spawn_async_task(async move {
                        let result = actor.upload_blob(bytes).await;
                        tx.try_send(result).ok();
                    });
                }
            }
        })
        .send()
        .await?;
    rx.recv().await?
}

async fn spawn_child_doc(api: &Api, doc: Arc<LoroDoc>, id: Hash) -> anyhow::Result<()> {
    let firewall = Firewall::for_child_doc(api.doc_id);
    FIREWALL_REGISTRY.write().insert(id, firewall.clone());
    if let Some(parent_space) = unavi_space::membership::doc_space(api.doc_id) {
        unavi_space::membership::DOC_SPACE_REGISTRY
            .write()
            .insert(id, parent_space);
    }
    AsyncCommands::default()
        .spawn((Hsd(doc), HsdRecordId(id), firewall, api.permissions.clone()))
        .send()
        .await?;
    Ok(())
}

pub async fn self_prim(api: &Api) -> anyhow::Result<u32> {
    let mut scene = api.wired_scene.lock().await;
    Ok(scene.prims.insert(PrimRes {
        doc:      Arc::clone(&api.doc),
        doc_id:   api.doc_id,
        id:       api.prim,
        is_proxy: false,
    }))
}

pub async fn self_document(api: &Api) -> anyhow::Result<u32> {
    let mut scene = api.wired_scene.lock().await;
    Ok(scene.docs.insert(DocRes {
        doc: Arc::clone(&api.doc),
        id:  api.doc_id,
    }))
}

pub async fn get_document(api: &Api, id: Vec<u8>) -> anyhow::Result<Option<u32>> {
    let id = Hash::from_slice(&id)?;
    validate_firewall(&api.doc_id, &id, Channel::SceneRead)?;

    let mut scene = api.wired_scene.lock().await;

    if let Some(key) = scene
        .docs
        .items
        .iter()
        .find(|(_, v)| v.id == id)
        .map(|(k, _)| *k)
    {
        return Ok(scene.docs.insert_clone(key));
    }

    if id == api.doc_id {
        return Ok(Some(scene.docs.insert(DocRes {
            doc: Arc::clone(&api.doc),
            id,
        })));
    }

    let (tx, rx) = async_channel::bounded::<Option<Arc<LoroDoc>>>(1);
    AsyncCommands::default()
        .push(move |world: &mut World| {
            let doc = world
                .query::<(&HsdRecordId, &Hsd)>()
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
    Ok(Some(scene.docs.insert(DocRes { doc, id })))
}

pub async fn remove_document(api: &Api, id: Vec<u8>) -> anyhow::Result<()> {
    let id = Hash::from_slice(&id)?;
    if let Err(err) = validate_firewall(&api.doc_id, &id, Channel::SceneWrite) {
        debug!(?err, "remove_document denied by firewall, skipping");
        return Ok(());
    }

    let mut scene = api.wired_scene.lock().await;
    let key = scene
        .docs
        .items
        .iter()
        .find(|(_, v)| v.id == id)
        .map(|(k, _)| *k)
        .ok_or_else(|| anyhow::anyhow!("resource not found"))?;
    let Some(doc) = scene.docs.remove(key) else {
        return Ok(());
    };
    drop(scene);

    AsyncCommands::default()
        .push(move |world: &mut World| {
            let mut query = world.query::<(Entity, &HsdRecordId)>();
            if let Some((entity, _)) = query.iter(world).find(|(_, v)| v.0 == doc.id) {
                world.despawn(entity);
            }
        })
        .send()
        .await?;

    Ok(())
}

pub async fn load_hsd(api: &Api, blob_id: Vec<u8>) -> anyhow::Result<u32> {
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

    let id = {
        let (mut write, rx, _cancel) = WriteRecord::new(None);
        write.schemas = vec![SchemaDef {
            schema:    (&*SCHEMA_HSD).into(),
            container: "hsd".into(),
            f:         Arc::new(move |doc| {
                file.load_into_doc(doc)?;
                Ok(())
            }),
        }];
        AsyncCommands::default().trigger(write).send().await?;
        rx.recv().await?
    };

    let doc = {
        let (read, rx, _cancel) = ReadRecord::new(id);
        AsyncCommands::default().trigger(read).send().await?;
        Arc::new(rx.recv().await?)
    };

    spawn_child_doc(api, Arc::clone(&doc), id).await?;

    let mut scene = api.wired_scene.lock().await;
    Ok(scene.docs.insert(DocRes { doc, id }))
}

pub async fn publish_document(api: &Api, id: Vec<u8>) -> anyhow::Result<()> {
    let id = Hash::from_slice(&id)?;
    validate_firewall(&api.doc_id, &id, Channel::SceneWrite)?;

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
        let (tx, rx) = async_channel::bounded::<Option<Hash>>(1);
        AsyncCommands::default()
            .push(move |world: &mut World| {
                let active = world
                    .get_resource::<unavi_space::anchor::ActiveSpace>()
                    .and_then(|a| a.0);
                let hash = active.and_then(|e| world.get::<unavi_space::Space>(e).map(|s| s.0));
                tx.try_send(hash).ok();
            })
            .send()
            .await?;
        rx.recv()
            .await?
            .ok_or_else(|| anyhow::anyhow!("doc has no space and no active space"))?
    };

    if !unavi_space::state::doc::add_doc(space, id) {
        anyhow::bail!("space state not tracked locally");
    }

    Ok(())
}

pub async fn create_document(api: &Api) -> anyhow::Result<u32> {
    let id = {
        let (mut write, rx, cancel) = WriteRecord::new(None);
        write.schemas = vec![SchemaDef {
            schema:    (&*SCHEMA_HSD).into(),
            container: "hsd".into(),
            f:         Arc::new(|doc| {
                // Ensure the HSD tree container exists.
                let _ = doc.get_tree(&*HSD_CONTAINER_ID);
                Ok(())
            }),
        }];
        AsyncCommands::default().trigger(write).send().await?;
        let id = rx.recv().await?;
        drop(cancel);
        id
    };

    let doc = {
        let (read, rx, cancel) = ReadRecord::new(id);
        AsyncCommands::default().trigger(read).send().await?;
        let doc = rx.recv().await?;
        drop(cancel);
        Arc::new(doc)
    };

    spawn_child_doc(api, Arc::clone(&doc), id).await?;

    let mut scene = api.wired_scene.lock().await;
    Ok(scene.docs.insert(DocRes { doc, id }))
}
