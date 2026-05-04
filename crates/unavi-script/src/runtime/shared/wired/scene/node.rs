use blake3::Hash;
use loro::TreeID;

use crate::runtime::shared::RuntimeBackend;

#[derive(Clone)]
pub struct NodeRes {
    pub id: TreeID,
    pub doc_id: Hash,
}

pub fn node_clone(backend: &RuntimeBackend, rep: u32) -> anyhow::Result<u32> {
    backend
        .wired_scene
        .try_lock()?
        .nodes
        .insert_clone(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid node"))
}

pub fn node_drop(backend: &RuntimeBackend, rep: u32) -> anyhow::Result<()> {
    backend.wired_scene.try_lock()?.nodes.remove(rep);
    Ok(())
}
