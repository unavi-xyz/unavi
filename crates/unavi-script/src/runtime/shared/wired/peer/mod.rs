use blake3::Hash;
use unavi_space::{
    membership::doc_space,
    peer::{
        self_did as space_self_did,
        self_peer_id,
    },
    state::replicas,
};

use crate::runtime::shared::Api;

#[must_use]
pub fn self_peer(_api: &Api) -> Option<Vec<u8>> {
    self_peer_id().map(|p| p.to_vec())
}

#[must_use]
pub fn self_did(_api: &Api) -> Option<String> {
    space_self_did()
}

#[must_use]
pub fn doc_owner(_api: &Api, doc_id: Vec<u8>) -> Option<Vec<u8>> {
    let bytes = <[u8; 32]>::try_from(doc_id.as_slice()).ok()?;
    let doc = Hash::from(bytes);
    let space = doc_space(doc)?;
    replicas::owner(space, doc).map(|p| p.to_vec())
}

#[must_use]
pub fn is_self_owner(api: &Api) -> bool {
    let Some(space) = doc_space(api.doc_id) else {
        return false;
    };
    replicas::is_self_owner(space, api.doc_id)
}
