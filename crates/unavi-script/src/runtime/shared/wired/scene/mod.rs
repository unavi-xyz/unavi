use std::sync::Arc;

use bevy::{ecs::world::CommandQueue, prelude::*};
use bevy_hsd::{HsdDoc, HsdRecordId};
use bevy_wds::record::{
    read::ReadRecord,
    write::{SchemaDef, WriteRecord},
};
use blake3::Hash;
use loro::{LoroDoc, TreeID};
use unavi_util::async_commands::ASYNC_COMMAND_QUEUE;
use wired_schemas::SCHEMA_HSD;

use crate::runtime::shared::{
    slot_map::SlotMap,
    wired::scene::{doc::DocRes, node::NodeRes},
};

pub mod doc;
pub mod node;

pub struct SceneContext {
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

            let doc = rx.recv().await?;
            Arc::new(doc)
        };

        let mut q = CommandQueue::default();
        q.push(bevy::ecs::system::command::spawn_batch([(
            HsdDoc(Arc::clone(&doc)),
            HsdRecordId(id),
        )]));
        ASYNC_COMMAND_QUEUE.0.try_send(q)?;

        let res = DocRes { id };
        let rep = self.docs.insert(res);
        Ok(rep)
    }

    // pub fn remove_document_by_rep(&mut self, handle: u32) {
    //     let Some(doc) = self.docs.remove(handle) else {
    //         return;
    //     };
    //     let mut q = CommandQueue::default();
    //     q.push(DespawnByDoc(doc));
    //     let _ = ASYNC_COMMAND_QUEUE.0.try_send(q);
    // }
}

struct DespawnByDoc(Arc<LoroDoc>);

impl Command for DespawnByDoc {
    fn apply(self, world: &mut World) {
        let mut q = world.query::<(Entity, &HsdDoc)>();
        let to_despawn = q
            .iter(world)
            .find(|(_, hsd)| Arc::ptr_eq(&hsd.0, &self.0))
            .map(|(e, _)| e);
        if let Some(entity) = to_despawn {
            world.despawn(entity);
        }
    }
}
