use std::sync::Arc;

use bevy::{ecs::world::CommandQueue, prelude::*};
use bevy_hsd::{HsdDoc, HsdRecordId};
use bevy_wds::{
    blob::get::GetBlob,
    record::{
        read::ReadRecord,
        write::{SchemaDef, WriteRecord},
    },
};
use blake3::Hash;
use loro::{LoroDoc, TreeID};
use unavi_util::async_commands::ASYNC_COMMAND_QUEUE;
use wired_schemas::SCHEMA_HSD;

use crate::{
    firewall::Firewall,
    permissions::ApiPermissions,
    runtime::shared::{
        slot_map::SlotMap,
        wired::scene::{doc::DocRes, node::NodeRes},
    },
};

pub mod doc;
pub mod node;

pub struct SceneContext {
    pub perms: ApiPermissions,
    pub self_doc: Hash,
    pub self_node: TreeID,
}

pub struct WiredSceneBackend {
    pub ctx: SceneContext,
    pub docs: SlotMap<DocRes>,
    pub nodes: SlotMap<NodeRes>,
}

impl WiredSceneBackend {
    pub fn new(ctx: SceneContext) -> Self {
        Self {
            ctx,
            docs: SlotMap::default(),
            nodes: SlotMap::default(),
        }
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
        })
    }

    pub fn get_document(&mut self, id: Vec<u8>) -> Option<u32> {
        self.docs
            .items
            .iter()
            .find(|(_, v)| v.id.as_slice() == id)
            .map(|(k, _)| *k)
            .and_then(|key| self.docs.new_owned(key))
    }

    fn enqueue_spawn_child_doc(&self, q: &mut CommandQueue, doc: Arc<LoroDoc>, id: Hash) {
        let firewall = Firewall::for_child_doc(self.ctx.self_doc);
        let perms = self.ctx.perms.clone();
        q.push(bevy::ecs::system::command::spawn_batch([(
            HsdDoc(doc),
            HsdRecordId(id),
            firewall,
            perms,
        )]));
    }

    pub async fn load_hsd(&mut self, blob_id: Vec<u8>) -> anyhow::Result<u32> {
        let blob_hash = blake3::Hash::from_slice(&blob_id)?;

        // Fetch blob bytes.
        let bytes = {
            let (tx, rx) = async_channel::bounded(1);
            let get = GetBlob {
                hash: blob_hash,
                cancel: None,
                tx,
            };
            let mut q = CommandQueue::default();
            q.push(bevy::ecs::system::command::trigger(get));
            ASYNC_COMMAND_QUEUE.0.try_send(q)?;
            rx.recv().await?
        };

        // Create WDS record, importing blob as initial state.
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
            let mut q = CommandQueue::default();
            q.push(bevy::ecs::system::command::trigger(write));
            ASYNC_COMMAND_QUEUE.0.try_send(q)?;
            rx.recv().await?
        };

        // Read created record.
        let doc = {
            let (read, rx, _cancel) = ReadRecord::new(id);
            let mut q = CommandQueue::default();
            q.push(bevy::ecs::system::command::trigger(read));
            ASYNC_COMMAND_QUEUE.0.try_send(q)?;
            Arc::new(rx.recv().await?)
        };

        let mut q = CommandQueue::default();
        self.enqueue_spawn_child_doc(&mut q, doc, id);
        ASYNC_COMMAND_QUEUE.0.try_send(q)?;

        let rep = self.docs.insert(DocRes { id });
        Ok(rep)
    }

    pub async fn create_document(&mut self) -> anyhow::Result<u32> {
        // Create WDS record.
        let id = {
            let (mut write, rx, _cancel) = WriteRecord::new(None);
            write.schemas = vec![SchemaDef {
                schema: (&*SCHEMA_HSD).into(),
                container: "hsd".into(),
                f: Arc::new(|_| Ok(())),
            }];
            let mut q = CommandQueue::default();
            q.push(bevy::ecs::system::command::trigger(write));
            ASYNC_COMMAND_QUEUE.0.try_send(q)?;
            rx.recv().await?
        };

        // Read created record.
        let doc = {
            let (read, rx, _cancel) = ReadRecord::new(id);
            let mut q = CommandQueue::default();
            q.push(bevy::ecs::system::command::trigger(read));
            ASYNC_COMMAND_QUEUE.0.try_send(q)?;
            Arc::new(rx.recv().await?)
        };

        let mut q = CommandQueue::default();
        self.enqueue_spawn_child_doc(&mut q, doc, id);
        ASYNC_COMMAND_QUEUE.0.try_send(q)?;

        let rep = self.docs.insert(DocRes { id });
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
        let mut q = CommandQueue::default();
        q.push(move |world: &mut World| {
            let mut query = world.query::<(Entity, &HsdRecordId)>();
            if let Some((entity, _)) = query.iter(world).find(|(_, v)| v.0 == doc.id) {
                world.despawn(entity);
            }
        });
        let _ = ASYNC_COMMAND_QUEUE.0.try_send(q);
    }
}
