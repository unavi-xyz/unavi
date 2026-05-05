use blake3::Hash;
use smol_str::SmolStr;

use crate::runtime::shared::Api;

#[derive(Clone)]
pub struct MeshRes {
    pub id: SmolStr,
    pub doc_id: Hash,
}

pub fn mesh_clone(api: &Api, rep: u32) -> anyhow::Result<u32> {
    api.wired_scene
        .try_lock()?
        .meshes
        .insert_clone(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid mesh"))
}

pub fn mesh_drop(api: &Api, rep: u32) -> anyhow::Result<()> {
    api.wired_scene.try_lock()?.meshes.remove(rep);
    Ok(())
}
