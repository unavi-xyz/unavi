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
use unavi_util::async_commands::try_send_command;
use wired_schemas::SCHEMA_HSD;

use crate::{
    firewall::Firewall,
    permissions::ApiPermissions,
    registry::{OutboundTransform, TransformHandles},
    runtime::shared::{
        slot_map::SlotMap,
        wired::scene::{
            doc::DocRes, material::MaterialRes, mesh::MeshRes, node::NodeRes,
        },
    },
};

pub mod doc;
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
    pub(super) transform_registry: Arc<Mutex<HashMap<Hash, TransformHandles>>>,
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

    pub fn self_node(&mut self) -> u32 {
        self.nodes.insert(NodeRes {
            id: self.ctx.self_node,
            doc_id: self.ctx.self_doc,
        })
    }

    pub fn self_document(&mut self) -> u32 {
        self.docs.insert(DocRes {
            id: self.ctx.self_doc,
            transforms: self.handles_for(&self.ctx.self_doc),
        })
    }

    pub fn get_document(&mut self, id: Vec<u8>) -> Option<u32> {
        let hash = blake3::Hash::from_slice(&id).ok()?;
        let transforms = self
            .transform_registry
            .lock()
            .expect("registry poisoned")
            .get(&hash)
            .cloned()?;
        let existing_key = self
            .docs
            .items
            .iter()
            .find(|(_, v)| v.id == hash)
            .map(|(k, _)| *k);
        if let Some(key) = existing_key {
            self.docs.new_owned(key)
        } else {
            Some(self.docs.insert(DocRes { id: hash, transforms }))
        }
    }

    fn send_spawn_child_doc(
        &self,
        doc: Arc<LoroDoc>,
        id: Hash,
        transforms: TransformHandles,
    ) -> anyhow::Result<()> {
        let firewall = Firewall::for_child_doc(self.ctx.self_doc);
        let perms = self.ctx.perms.clone();
        try_send_command(bevy::ecs::system::command::spawn_batch([(
            HsdDoc(doc),
            HsdRecordId(id),
            firewall,
            perms,
            OutboundTransform(transforms),
        )]))?;
        Ok(())
    }

    pub async fn load_hsd(&mut self, blob_id: Vec<u8>) -> anyhow::Result<u32> {
        let blob_hash = blake3::Hash::from_slice(&blob_id)?;

        let bytes = {
            let (tx, rx) = async_channel::bounded(1);
            try_send_command(bevy::ecs::system::command::trigger(GetBlob {
                hash: blob_hash,
                cancel: None,
                tx,
            }))?;
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
            try_send_command(bevy::ecs::system::command::trigger(write))?;
            rx.recv().await?
        };

        let doc = {
            let (read, rx, _cancel) = ReadRecord::new(id);
            try_send_command(bevy::ecs::system::command::trigger(read))?;
            Arc::new(rx.recv().await?)
        };

        let transforms = TransformHandles::default();
        self.transform_registry
            .lock()
            .expect("registry poisoned")
            .insert(id, transforms.clone());

        self.send_spawn_child_doc(doc, id, transforms.clone())?;

        let rep = self.docs.insert(DocRes { id, transforms });
        Ok(rep)
    }

    pub async fn create_document(&mut self) -> anyhow::Result<u32> {
        let id = {
            let (mut write, rx, _cancel) = WriteRecord::new(None);
            write.schemas = vec![SchemaDef {
                schema: (&*SCHEMA_HSD).into(),
                container: "hsd".into(),
                f: Arc::new(|_| Ok(())),
            }];
            try_send_command(bevy::ecs::system::command::trigger(write))?;
            rx.recv().await?
        };

        let doc = {
            let (read, rx, _cancel) = ReadRecord::new(id);
            try_send_command(bevy::ecs::system::command::trigger(read))?;
            Arc::new(rx.recv().await?)
        };

        let transforms = TransformHandles::default();
        self.transform_registry
            .lock()
            .expect("registry poisoned")
            .insert(id, transforms.clone());

        self.send_spawn_child_doc(doc, id, transforms.clone())?;

        let rep = self.docs.insert(DocRes { id, transforms });
        Ok(rep)
    }

    pub fn remove_document(&mut self, id: Vec<u8>) {
        let Some(key) = self
            .docs
            .items
            .iter()
            .find(|(_, v)| v.id.as_slice() == id)
            .map(|(k, _)| *k)
        else {
            return;
        };
        let Some(doc) = self.docs.remove(key) else {
            return;
        };
        let _ = try_send_command(move |world: &mut World| {
            let mut query = world.query::<(Entity, &HsdRecordId)>();
            if let Some((entity, _)) = query.iter(world).find(|(_, v)| v.0 == doc.id) {
                world.despawn(entity);
            }
        });
    }
}
