use std::sync::Arc;

use bevy::prelude::*;
use bevy_hsd::{HsdDoc, HsdRecordId};
use bevy_wds::{
    blob::get::GetBlob,
    record::{
        read::ReadRecord,
        write::{SchemaDef, WriteRecord},
    },
};
use blake3::Hash;
use loro::LoroDoc;
use unavi_util::async_commands::AsyncCommands;
use wired_schemas::SCHEMA_HSD;

use crate::{
    firewall::{Channel, Firewall},
    runtime::shared::{
        Api,
        registry::firewall::validate_firewall,
        slot_map::SlotMap,
        wired::scene::{document::DocRes, material::MaterialRes, mesh::MeshRes, node::NodeRes},
    },
};

pub mod document;
pub mod material;
pub mod mesh;
pub mod node;

#[derive(Default)]
pub struct WiredSceneApi {
    pub docs: SlotMap<DocRes>,
    pub materials: SlotMap<MaterialRes>,
    pub meshes: SlotMap<MeshRes>,
    pub nodes: SlotMap<NodeRes>,
}

fn spawn_child_doc(api: &Api, doc: Arc<LoroDoc>, id: Hash) -> anyhow::Result<()> {
    let firewall = Firewall::for_child_doc(api.doc_id);
    AsyncCommands::default()
        .spawn((
            HsdDoc(doc),
            HsdRecordId(id),
            firewall,
            api.permissions.clone(),
        ))
        .try_send()?;
    Ok(())
}

pub fn self_node(api: &Api) -> anyhow::Result<u32> {
    let mut scene = api.wired_scene.try_lock()?;
    Ok(scene.nodes.insert(NodeRes {
        doc: Arc::clone(&api.doc),
        doc_id: api.doc_id,
        id: api.node,
    }))
}

pub fn self_document(api: &Api) -> anyhow::Result<u32> {
    let mut scene = api.wired_scene.try_lock()?;
    Ok(scene.docs.insert(DocRes {
        doc: Arc::clone(&api.doc),
        id: api.doc_id,
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
                .query::<(&HsdRecordId, &HsdDoc)>()
                .iter(world)
                .find(|(rid, _)| rid.0 == id)
                .map(|(_, d)| Arc::clone(&d.0));
            tx.try_send(doc).ok();
        })
        .try_send()?;

    let Some(doc) = rx.recv().await? else {
        return Ok(None);
    };
    Ok(Some(scene.docs.insert(DocRes { doc, id })))
}

pub fn remove_document(api: &Api, id: Vec<u8>) -> anyhow::Result<()> {
    let id = Hash::from_slice(&id)?;
    validate_firewall(&api.doc_id, &id, Channel::SceneWrite)?;

    let mut scene = api.wired_scene.try_lock()?;
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
        .try_send()?;

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

    let id = {
        let (mut write, rx, _cancel) = WriteRecord::new(None);
        write.schemas = vec![SchemaDef {
            schema: (&*SCHEMA_HSD).into(),
            container: "hsd".into(),
            f: Arc::new(move |doc| {
                doc.import(&bytes)?;
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

    spawn_child_doc(api, Arc::clone(&doc), id)?;

    let mut scene = api.wired_scene.lock().await;
    Ok(scene.docs.insert(DocRes { doc, id }))
}

pub async fn create_document(api: &Api) -> anyhow::Result<u32> {
    let id = {
        let (mut write, rx, _cancel) = WriteRecord::new(None);
        write.schemas = vec![SchemaDef {
            schema: (&*SCHEMA_HSD).into(),
            container: "hsd".into(),
            f: Arc::new(|_| Ok(())),
        }];
        AsyncCommands::default().trigger(write).send().await?;
        rx.recv().await?
    };

    let doc = {
        let (read, rx, _cancel) = ReadRecord::new(id);
        AsyncCommands::default().trigger(read).send().await?;
        Arc::new(rx.recv().await?)
    };

    spawn_child_doc(api, Arc::clone(&doc), id)?;

    let mut scene = api.wired_scene.lock().await;
    Ok(scene.docs.insert(DocRes { doc, id }))
}
