//! Which DID authors a document.
//!
//! A document's author is a DID that holds its namespace write key. An
//! [`Authorship`] proves that with two signatures over one claim naming the
//! namespace and the DID: the namespace key's, which only a writer can make,
//! and the DID's, which only the DID can make. Neither alone is enough, since a
//! writer could name any DID and a DID could name any namespace.
//!
//! A document carries its proof under [`AUTHORSHIP_KEY`], so a reader holding
//! only the namespace id can check it. Every writer can replace that entry, so
//! when several DIDs share a write key the latest claim names the author.

use iroh::Signature;
use iroh_docs::{
    NamespaceId,
    NamespacePublicKey,
    NamespaceSecret,
};
use serde::{
    Deserialize,
    Serialize,
};
use unavi_store::{
    Document,
    Store,
};
use xdid::core::did::Did;

use crate::{
    identity::Identity,
    resolver::Resolver,
    signed::{
        Signable,
        Signed,
        VerifyError,
    },
};

/// Where a document records its [`Authorship`].
pub const AUTHORSHIP_KEY: &str = "wired/authorship";

/// Context of the namespace key's signature, distinct from the DID's so one
/// can never stand in for the other.
const NAMESPACE_CONTEXT: &[u8] = b"wired/authorship/namespace\0";

/// Bytes an encoded [`Authorship`] may take. Far above any real one; it bounds
/// what a reader decodes from a peer-written entry.
pub const MAX_AUTHORSHIP_BYTES: usize = 4096;

#[derive(Debug, thiserror::Error)]
pub enum AuthorshipError {
    #[error("malformed authorship proof")]
    Malformed,
    #[error("authorship proof is for another namespace")]
    WrongNamespace,
    #[error("authorship proof is not signed by the namespace key")]
    NotSignedByNamespace,
    #[error(transparent)]
    NotSignedByDid(#[from] VerifyError),
    #[error("this store does not hold the namespace write key")]
    NotWritable,
    #[error(transparent)]
    Store(#[from] unavi_store::Error),
    #[error("signing failed: {0}")]
    Sign(anyhow::Error),
}

/// The claim both keys sign.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorshipClaim {
    pub ns:  NamespaceId,
    pub did: Did,
}

impl Signable for AuthorshipClaim {
    const SIGNING_CONTEXT: &'static str = "wired/authorship";
}

/// Proof that a DID authors a namespace. See the module docs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Authorship {
    claim:               Signed<AuthorshipClaim>,
    namespace_signature: Vec<u8>,
}

impl Authorship {
    /// Signs `identity` as the author of the namespace `write_key` opens.
    pub fn sign(write_key: &NamespaceSecret, identity: &Identity) -> Result<Self, AuthorshipError> {
        let claim = AuthorshipClaim {
            ns:  write_key.id(),
            did: identity.did().clone(),
        }
        .sign(identity.signing_key())
        .map_err(AuthorshipError::Sign)?;

        let namespace_signature = write_key.sign(&namespace_bytes(&claim)).to_bytes().to_vec();

        Ok(Self {
            claim,
            namespace_signature,
        })
    }

    /// The DID this proof names, checked against `ns`'s key only.
    ///
    /// Offline, but not a proof on its own: see [`Self::verify`].
    pub fn verify_namespace(&self, ns: NamespaceId) -> Result<Did, AuthorshipError> {
        let claim = self
            .claim
            .payload()
            .map_err(|_| AuthorshipError::Malformed)?;
        if claim.ns != ns {
            return Err(AuthorshipError::WrongNamespace);
        }

        let key = NamespacePublicKey::from_bytes(ns.as_bytes())
            .map_err(|_| AuthorshipError::Malformed)?;
        let signature = <[u8; 64]>::try_from(self.namespace_signature.as_slice())
            .map_err(|_| AuthorshipError::Malformed)?;
        key.verify(
            &namespace_bytes(&self.claim),
            &Signature::from_bytes(&signature),
        )
        .map_err(|_| AuthorshipError::NotSignedByNamespace)?;

        Ok(claim.did)
    }

    /// The DID that authors `ns`, once both signatures check out.
    pub async fn verify(
        &self,
        ns: NamespaceId,
        resolver: &Resolver,
    ) -> Result<Did, AuthorshipError> {
        let did = self.verify_namespace(ns)?;
        self.claim.verify_did(&did, resolver).await?;
        Ok(did)
    }
}

fn namespace_bytes(claim: &Signed<AuthorshipClaim>) -> Vec<u8> {
    let mut out = NAMESPACE_CONTEXT.to_vec();
    out.extend_from_slice(&claim.signing_bytes());
    out
}

/// Records `identity` as `doc`'s author, keeping a proof already naming it.
pub async fn claim(
    store: &Store,
    doc: &Document,
    identity: &Identity,
) -> Result<Authorship, AuthorshipError> {
    if let Some(existing) = recorded(doc).await?
        && existing
            .verify_namespace(doc.id())
            .is_ok_and(|did| &did == identity.did())
    {
        return Ok(existing);
    }

    let write_key = store
        .write_key(doc.id())
        .await?
        .ok_or(AuthorshipError::NotWritable)?;
    let proof = Authorship::sign(&write_key, identity)?;

    let bytes = postcard::to_stdvec(&proof).map_err(|_| AuthorshipError::Malformed)?;
    doc.set(AUTHORSHIP_KEY, bytes).await?;
    Ok(proof)
}

/// The proof `doc` records, unverified. `None` when it records none or its
/// content has not arrived.
pub async fn recorded(doc: &Document) -> Result<Option<Authorship>, AuthorshipError> {
    let Some(entry) = doc
        .entry(AUTHORSHIP_KEY, unavi_store::Tombstones::Exclude)
        .await?
    else {
        return Ok(None);
    };
    if entry.content_len() > MAX_AUTHORSHIP_BYTES as u64 {
        return Err(AuthorshipError::Malformed);
    }
    let Some(bytes) = doc.value(&entry).await? else {
        return Ok(None);
    };
    postcard::from_bytes(&bytes)
        .map(Some)
        .map_err(|_| AuthorshipError::Malformed)
}

/// The DID that authors `doc`. `None` when it records no proof, or one that
/// does not verify.
pub async fn author_of(doc: &Document, resolver: &Resolver) -> Option<Did> {
    let proof = recorded(doc).await.ok().flatten()?;
    proof.verify(doc.id(), resolver).await.ok()
}

#[cfg(test)]
mod tests {
    use rand::RngCore;
    use xdid::method::key::{
        DidKeyPair,
        PublicKey,
        p256::P256KeyPair,
    };

    use super::*;

    fn identity() -> Identity {
        let key = P256KeyPair::generate();
        let did = key.public().to_did();
        Identity::new(did, key)
    }

    fn namespace() -> NamespaceSecret {
        let mut bytes = [0u8; 32];
        rand::rng().fill_bytes(&mut bytes);
        NamespaceSecret::from_bytes(&bytes)
    }

    #[tokio::test]
    async fn a_writer_proves_its_did() {
        let (ns, me) = (namespace(), identity());
        let proof = Authorship::sign(&ns, &me).expect("sign");

        let did = proof
            .verify(ns.id(), &Resolver::new().expect("resolver"))
            .await
            .expect("both signatures verify");
        assert_eq!(&did, me.did());
    }

    #[test]
    fn a_proof_does_not_carry_to_another_namespace() {
        let proof = Authorship::sign(&namespace(), &identity()).expect("sign");

        assert!(matches!(
            proof.verify_namespace(namespace().id()),
            Err(AuthorshipError::WrongNamespace)
        ));
    }

    #[tokio::test]
    async fn a_writer_cannot_name_a_did_it_does_not_hold() {
        let (ns, me, victim) = (namespace(), identity(), identity());
        let mut proof = Authorship::sign(&ns, &me).expect("sign");

        // The victim's DID under the writer's identity key.
        let forged = Identity::new(victim.did().clone(), me.signing_key().clone());
        proof.claim = AuthorshipClaim {
            ns:  ns.id(),
            did: victim.did().clone(),
        }
        .sign(forged.signing_key())
        .expect("sign");
        proof.namespace_signature = ns.sign(&namespace_bytes(&proof.claim)).to_bytes().to_vec();

        assert!(
            proof
                .verify(ns.id(), &Resolver::new().expect("resolver"))
                .await
                .is_err(),
            "holding a write key must not let a writer attribute it to anyone"
        );
    }

    #[test]
    fn a_did_cannot_claim_a_namespace_it_cannot_write() {
        let (ns, me) = (namespace(), identity());
        let mut proof = Authorship::sign(&namespace(), &me).expect("sign");
        proof.claim = AuthorshipClaim {
            ns:  ns.id(),
            did: me.did().clone(),
        }
        .sign(me.signing_key())
        .expect("sign");

        assert!(matches!(
            proof.verify_namespace(ns.id()),
            Err(AuthorshipError::NotSignedByNamespace)
        ));
    }
}
