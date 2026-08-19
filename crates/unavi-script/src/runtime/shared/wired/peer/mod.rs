use hsd::id::DocId;
use iroh_docs::NamespaceId;
use unavi_policy::check::space_of;
use unavi_space::{
    peer::{
        self_did as space_self_did,
        self_peer_id,
    },
    state::replicas,
};

use crate::runtime::shared::Api;

/// Replicas key by 32 opaque bytes, which a document id equally is.
fn ns(id: DocId) -> NamespaceId {
    NamespaceId::from(&id.0)
}

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
    let doc = DocId(bytes);
    let space = space_of(doc)?;
    replicas::owner(ns(space), ns(doc)).map(|p| p.to_vec())
}

#[must_use]
pub fn is_self_owner(api: &Api) -> bool {
    let Some(space) = space_of(api.doc_id) else {
        return false;
    };
    replicas::is_self_owner(ns(space), ns(api.doc_id))
}
