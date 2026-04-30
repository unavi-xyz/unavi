use blake3::Hash;
use loro::TreeID;

#[derive(Clone)]
pub struct NodeRes {
    pub id: TreeID,
    pub doc_id: Hash,
}
