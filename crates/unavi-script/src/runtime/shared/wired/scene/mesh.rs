use blake3::Hash;
use smol_str::SmolStr;

use crate::runtime::shared::RuntimeBackend;

#[derive(Clone)]
pub struct MeshRes {
    pub id: SmolStr,
    pub doc_id: Hash,
}

pub fn mesh_clone(backend: &RuntimeBackend, rep: u32) -> anyhow::Result<u32> {
    backend
        .wired_scene
        .try_lock()?
        .meshes
        .insert_clone(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid mesh"))
}

pub fn mesh_drop(backend: &RuntimeBackend, rep: u32) -> anyhow::Result<()> {
    backend.wired_scene.try_lock()?.meshes.remove(rep);
    Ok(())
}
