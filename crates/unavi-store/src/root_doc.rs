//! This node's root document: the one namespace everything else hangs off.

use iroh_docs::{
    NamespaceId,
    protocol::Docs,
};
use parking_lot::RwLock;

use crate::{
    local::Storage,
    namespace,
};

/// Where the id is recorded, beside the identity that authored it.
const KEY: &str = "root-doc";

static ROOT_DOC: RwLock<Option<NamespaceId>> = RwLock::new(None);

pub fn set_root_doc(ns: NamespaceId) {
    *ROOT_DOC.write() = Some(ns);
}

#[must_use]
pub fn root_doc() -> Option<NamespaceId> {
    *ROOT_DOC.read()
}

/// Opens this node's root document, minting it on first use.
pub async fn open_or_mint(docs: &Docs, storage: &Storage) -> anyhow::Result<NamespaceId> {
    namespace::open_or_mint(docs, storage, KEY).await
}
