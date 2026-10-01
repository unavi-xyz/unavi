//! The document a DID's own data hangs off.
//!
//! Ordinary in every way a store cares about. What makes it a root is only
//! that its keys are agreed on, so a peer handed its id can read a profile or
//! an avatar out of it without being told the layout.

use unavi_store::{
    Store,
    document::Document,
};

/// Storage key recording this device's root document id.
///
/// Recorded rather than derived from the identity key. A namespace id computed
/// from a secret is one nobody else can compute, and the whole point of a root
/// document is that others can read it.
const KEY: &str = "root-doc.bin";

/// Opens this device's root document, minting one on first use.
pub async fn open(store: &Store) -> anyhow::Result<Document> {
    store.open_named_doc(KEY).await
}
