use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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
    permissions::ApiPermissions,
    registry::{OutboundTransform, TransformHandles},
    runtime::shared::{
        RuntimeBackend,
        slot_map::SlotMap,
        wired::scene::{document::DocRes, material::MaterialRes, mesh::MeshRes, node::NodeRes},
    },
};

pub mod document;
pub mod material;
pub mod mesh;
pub mod node;

pub struct SceneContext {
    pub perms: ApiPermissions,
    pub self_doc: Hash,
    pub self_node: loro::TreeID,
}

pub struct WiredSceneBackend {
    pub ctx: SceneContext,
    pub docs: SlotMap<DocRes>,
    pub materials: SlotMap<MaterialRes>,
    pub meshes: SlotMap<MeshRes>,
    pub nodes: SlotMap<NodeRes>,
    pub transform_registry: Arc<Mutex<HashMap<Hash, TransformHandles>>>,
}

impl WiredSceneBackend {
    pub fn new(
        ctx: SceneContext,
        transform_registry: Arc<Mutex<HashMap<Hash, TransformHandles>>>,
    ) -> Self {
        Self {
            ctx,
            docs: SlotMap::default(),
            materials: SlotMap::default(),
            meshes: SlotMap::default(),
            nodes: SlotMap::default(),
            transform_registry,
        }
    }

    fn handles_for(&self, id: &Hash) -> TransformHandles {
        self.transform_registry
            .lock()
            .expect("registry poisoned")
            .get(id)
            .cloned()
            .unwrap_or_default()
    }

    fn spawn_child_doc(
        &self,
        doc: Arc<LoroDoc>,
        id: Hash,
        transforms: TransformHandles,
    ) -> anyhow::Result<()> {
        let firewall = Firewall::for_child_doc(self.ctx.self_doc);
        let perms = self.ctx.perms.clone();

        AsyncCommands::default()
            .spawn((
                HsdDoc(doc),
                HsdRecordId(id),
                firewall,
                perms,
                OutboundTransform(transforms),
            ))
            .try_send()?;

        Ok(())
    }
}

pub fn self_node(backend: &RuntimeBackend) -> anyhow::Result<u32> {
    let mut scene = backend.wired_scene.try_lock()?;
    let id = scene.ctx.self_node;
    let doc_id = scene.ctx.self_doc;
    Ok(scene.nodes.insert(NodeRes { id, doc_id }))
}

pub fn self_document(backend: &RuntimeBackend) -> anyhow::Result<u32> {
    let mut scene = backend.wired_scene.try_lock()?;
    let id = scene.ctx.self_doc;
    let transforms = scene.handles_for(&id);
    Ok(scene.docs.insert(DocRes { id, transforms }))
}

pub fn get_document(backend: &RuntimeBackend, id: Vec<u8>) -> anyhow::Result<Option<u32>> {
    let hash = Hash::from_slice(&id)?;
    let mut scene = backend.wired_scene.try_lock()?;

    let Some(transforms) = scene
        .transform_registry
        .lock()
        .expect("lock")
        .get(&hash)
        .cloned()
    else {
        return Ok(None);
    };

    if let Some(key) = scene
        .docs
        .items
        .iter()
        .find(|(_, v)| v.id == hash)
        .map(|(k, _)| *k)
    {
        Ok(scene.docs.insert_clone(key))
    } else {
        Ok(Some(scene.docs.insert(DocRes {
            id: hash,
            transforms,
        })))
    }
}

pub fn remove_document(backend: &RuntimeBackend, id: Vec<u8>) -> anyhow::Result<()> {
    let mut scene = backend.wired_scene.try_lock()?;
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

pub async fn load_hsd(backend: &RuntimeBackend, blob_id: Vec<u8>) -> anyhow::Result<u32> {
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

    let transforms = TransformHandles::default();
    let mut scene = backend.wired_scene.lock().await;
    scene
        .transform_registry
        .lock()
        .expect("registry poisoned")
        .insert(id, transforms.clone());
    scene.spawn_child_doc(doc, id, transforms.clone())?;

    Ok(scene.docs.insert(DocRes { id, transforms }))
}

pub async fn create_document(backend: &RuntimeBackend) -> anyhow::Result<u32> {
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

    let transforms = TransformHandles::default();
    let mut scene = backend.wired_scene.lock().await;
    scene
        .transform_registry
        .lock()
        .expect("registry poisoned")
        .insert(id, transforms.clone());
    scene.spawn_child_doc(doc, id, transforms.clone())?;

    Ok(scene.docs.insert(DocRes { id, transforms }))
}
