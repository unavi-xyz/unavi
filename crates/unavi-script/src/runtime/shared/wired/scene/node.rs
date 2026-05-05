use blake3::Hash;
use loro::TreeID;

use crate::runtime::shared::Api;

#[derive(Clone)]
pub struct NodeRes {
    pub id: TreeID,
    pub document: Hash,
}

pub fn clone(api: &Api, rep: u32) -> anyhow::Result<u32> {
    api.wired_scene
        .try_lock()?
        .nodes
        .insert_clone(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid node"))
}

pub fn drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_scene.try_lock()?.nodes.remove(rep);
    Ok(())
}
