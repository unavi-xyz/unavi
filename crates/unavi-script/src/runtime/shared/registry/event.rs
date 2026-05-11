use std::{
    collections::HashMap,
    sync::{Arc, LazyLock},
};

use async_channel::Sender;
use blake3::Hash;
use parking_lot::RwLock;

use crate::runtime::shared::registry::transform::AbsoluteNodeId;

pub static EVENT_RECEPTOR_REGISTRY: LazyLock<RwLock<HashMap<u32, ReceptorEntry>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

pub struct ReceptorEntry {
    pub channels: Vec<String>,
    pub doc_id: Hash,
    pub scope: ReceptorScope,
    pub source_documents: Option<Vec<Vec<u8>>>,
    pub tx: Sender<InboundEvent>,
}

pub enum ReceptorScope {
    Global,
    Spatial { node: AbsoluteNodeId, radius: f32 },
}

pub struct InboundEvent {
    pub channel: String,
    pub payload: Arc<Vec<u8>>,
    pub sender_document: Vec<u8>,
    pub sender_scope: SenderScope,
    pub time: u64,
}

pub enum SenderScope {
    Global,
    Spatial { distance: f32, node: AbsoluteNodeId },
}
