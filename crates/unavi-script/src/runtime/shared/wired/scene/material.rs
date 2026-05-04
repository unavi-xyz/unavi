use blake3::Hash;
use smol_str::SmolStr;

use crate::runtime::shared::RuntimeBackend;

#[derive(Clone)]
pub struct MaterialRes {
    pub id: SmolStr,
    pub doc_id: Hash,
}

pub fn material_clone(backend: &RuntimeBackend, rep: u32) -> anyhow::Result<u32> {
    backend
        .wired_scene
        .try_lock()?
        .materials
        .insert_clone(rep)
        .ok_or_else(|| anyhow::anyhow!("invalid material"))
}

pub fn material_drop(backend: &RuntimeBackend, rep: u32) -> anyhow::Result<()> {
    backend.wired_scene.try_lock()?.materials.remove(rep);
    Ok(())
}
