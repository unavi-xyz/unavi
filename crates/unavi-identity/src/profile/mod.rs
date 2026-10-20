//! A user's public profile document, and how a reader finds it from a DID.
//!
//! # Namespace
//!
//! The namespace secret is derived from the identity key, so every device a
//! user owns opens and writes the same profile. Display name and description
//! are the profile's `DocMeta`.
//!
//! # Finding One
//!
//! A `did:web` document names it in a [`crate::did_document::profile_service`].
//! Any other DID is asked over [`protocol::ALPN`], which answers with an
//! [`Authorship`] that [`Authorship::verify_for`] checks against the DID.

use iroh_docs::{
    NamespaceId,
    NamespaceSecret,
};
use p256::pkcs8::DecodePrivateKey;
use unavi_store::{
    Document,
    Store,
};
use xdid::method::key::DidKeyPair;
use zeroize::Zeroizing;

use crate::{
    authorship::{
        self,
        Authorship,
        AuthorshipError,
    },
    identity::Identity,
};

pub mod protocol;

const DERIVE_CONTEXT: &str = "wired/profile/namespace";

#[derive(Debug, thiserror::Error)]
#[error("the identity key could not be encoded")]
pub struct KeyEncodingError;

/// The write key of a user's profile.
pub struct ProfileSecret(NamespaceSecret);

impl std::fmt::Debug for ProfileSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfileSecret")
            .field("id", &self.id())
            .finish_non_exhaustive()
    }
}

impl ProfileSecret {
    /// The profile secret of `identity`, derived from its private scalar so
    /// no key encoding can move it. Rotating the identity key moves the
    /// profile to a new namespace.
    pub fn derive(identity: &Identity) -> Result<Self, KeyEncodingError> {
        let pem = identity
            .signing_key()
            .to_pkcs8_pem()
            .map_err(|_| KeyEncodingError)?;
        let scalar = p256::SecretKey::from_pkcs8_pem(&pem).map_err(|_| KeyEncodingError)?;
        let scalar = Zeroizing::new(<[u8; 32]>::from(scalar.to_bytes()));
        let bytes = Zeroizing::new(blake3::derive_key(DERIVE_CONTEXT, scalar.as_slice()));
        Ok(Self(NamespaceSecret::from_bytes(&bytes)))
    }

    #[must_use]
    pub fn id(&self) -> NamespaceId {
        self.0.id()
    }

    /// Opens the profile writable in `store`, records `identity` as its
    /// author and serves it, returning the proof that
    /// [`protocol::ProfileProtocol`] answers with.
    pub async fn open(
        &self,
        store: &Store,
        identity: &Identity,
    ) -> Result<(Document, Authorship), AuthorshipError> {
        let doc = store.hold_writable(self.0.clone()).await?;
        let proof = authorship::claim(store, &doc, identity).await?;
        doc.serve().await?;
        Ok((doc, proof))
    }
}

#[cfg(test)] mod tests;
