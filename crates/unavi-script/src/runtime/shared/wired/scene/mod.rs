use std::sync::Arc;

use bevy::prelude::*;
use bevy_hsd::HsdDoc;
use blake3::Hash;
use loro::{LoroDoc, TreeID};

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

    pub fn create_document(&mut self) -> Result<Arc<LoroDoc>, String> {
        todo!()
        // let doc = Arc::new(LoroDoc::new());
        // let doc_for_spawn = Arc::clone(&doc);
        // let record_id = blake3::Hash::from_bytes(rand::random());
        //
        // let mut q = CommandQueue::default();
        // q.push(move |world: &mut World| {
        //     world.spawn((HsdDoc(doc_for_spawn), HsdRecordId(record_id)));
        // });
        // ASYNC_COMMAND_QUEUE
        //     .0
        //     .try_send(q)
        //     .map_err(|e| e.to_string())?;
        //
        // Ok(doc)
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
