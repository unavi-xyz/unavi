use blake3::Hash;
use smol_str::SmolStr;

#[derive(Clone)]
pub struct MeshRes {
    pub id: SmolStr,
    pub doc_id: Hash,
}
