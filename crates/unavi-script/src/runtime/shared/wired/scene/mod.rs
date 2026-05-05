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
    firewall::Firewall,
    runtime::shared::{
        Api,
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
    let firewall = Firewall::for_child_doc(api.document);
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
        id: api.node,
        document: api.document,
    }))
}

pub fn self_document(api: &Api) -> anyhow::Result<u32> {
    let mut scene = api.wired_scene.try_lock()?;
    Ok(scene.docs.insert(DocRes { id: api.document }))
}

pub fn get_document(api: &Api, id: Vec<u8>) -> anyhow::Result<Option<u32>> {
    let id = Hash::from_slice(&id)?;
    let mut scene = api.wired_scene.try_lock()?;

    if let Some(key) = scene
        .docs
        .items
        .iter()
        .find(|(_, v)| v.id == id)
        .map(|(k, _)| *k)
    {
        Ok(scene.docs.insert_clone(key))
    } else {
        Ok(Some(scene.docs.insert(DocRes { id })))
    }
}

pub fn remove_document(api: &Api, id: Vec<u8>) -> anyhow::Result<()> {
    let mut scene = api.wired_scene.try_lock()?;
    let key = scene
        .docs
        .items
        .iter()
        .find(|(_, v)| v.id.as_slice() == id)
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
    let blob_hash = blake3::Hash::from_slice(&blob_id)?;

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

    spawn_child_doc(api, doc, id)?;

    let mut scene = api.wired_scene.lock().await;
    Ok(scene.docs.insert(DocRes { id }))
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

    spawn_child_doc(api, doc, id)?;

    let mut scene = api.wired_scene.lock().await;
    Ok(scene.docs.insert(DocRes { id }))
}
