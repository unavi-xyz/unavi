//! The document a DID's own data hangs off.
//!
//! Ordinary in every way a store cares about. What makes it a root is only
//! that its keys are agreed on, so a peer handed its id can read a profile or
//! an avatar out of it without being told the layout.

use unavi_store::{
    Document,
    Store,
};

use crate::{
    authorship::{
        self,
        AuthorshipError,
    },
    identity::Identity,
};

/// The name this device records its root document id under.
///
/// Recorded rather than derived from the identity key. A namespace id computed
/// from a secret is one nobody else can compute, and the whole point of a root
/// document is that others can read it.
const NAME: &str = "root-doc";

/// Opens this device's root document, minting one on first use, and records
/// `identity` as its author.
pub async fn open(store: &Store, identity: &Identity) -> Result<Document, AuthorshipError> {
    let doc = store.named(NAME).await?;
    authorship::claim(store, &doc, identity).await?;
    Ok(doc)
}
