use hsd::id::DocId;

use crate::runtime::shared::Api;

#[must_use]
pub fn self_peer(api: &Api) -> Option<Vec<u8>> {
    Some(api.view.me().to_vec())
}

#[must_use]
pub fn self_did(api: &Api) -> Option<String> {
    Some(api.view.did())
}

#[must_use]
pub fn doc_owner(api: &Api, doc_id: Vec<u8>) -> Option<Vec<u8>> {
    let bytes = <[u8; 32]>::try_from(doc_id.as_slice()).ok()?;
    let doc = DocId(bytes);
    let space = api.view.space_of(doc)?;
    api.view.replicas().owner(space, doc).map(|p| p.to_vec())
}

#[must_use]
pub fn is_self_owner(api: &Api) -> bool {
    let me = api.view.me();
    let Some(space) = api.view.space_of(api.doc_id) else {
        return false;
    };
    api.view.replicas().is_owner(space, api.doc_id, me)
}
