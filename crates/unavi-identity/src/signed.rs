//! Payloads signed under a per-type context, by a DID or by an endpoint key.

use std::{
    convert::Infallible,
    marker::PhantomData,
};

use iroh::{
    EndpointId,
    Signature,
};
use iroh_docs::{
    NamespaceId,
    NamespacePublicKey,
    NamespaceSecret,
};
use serde::{
    Deserialize,
    Serialize,
};
use xdid::{
    core::{
        ResolutionError,
        did::Did,
        document::Document,
    },
    method::key::Signer,
};

use crate::{
    jwk,
    resolver::Resolver,
};

#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    #[error("could not resolve {did}: {source}")]
    Unresolvable {
        did:    Did,
        source: ResolutionError,
    },
    #[error("{0} lists no authentication key")]
    NoAuthenticationKey(Did),
    #[error("not signed by any authentication key of {0}")]
    NotSigned(Did),
    #[error("not signed by endpoint {0}")]
    NotSignedByEndpoint(EndpointId),
    #[error("not signed by namespace {0}")]
    NotSignedByNamespace(NamespaceId),
}

/// A `T` and a signature over it. Who signed is not recorded: the verifier
/// names the DID or endpoint it expects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signed<T>
where
    T: Signable,
{
    payload_bytes: Vec<u8>,
    signature:     Vec<u8>,
    _type:         PhantomData<T>,
}

impl<T> Signed<T>
where
    T: Signable,
{
    fn sign(payload: &T, key: &impl Signer) -> anyhow::Result<Self> {
        let payload_bytes = postcard::to_stdvec(payload)?;
        let signature = key.sign(&signing_bytes(T::SIGNING_CONTEXT, &payload_bytes))?;

        Ok(Self {
            payload_bytes,
            signature,
            _type: PhantomData,
        })
    }

    /// The exact bytes a signature covers. The payload alone is not signed.
    pub(crate) fn signing_bytes(&self) -> Vec<u8> {
        signing_bytes(T::SIGNING_CONTEXT, &self.payload_bytes)
    }

    /// The payload. Unverified until [`Self::verify_did`] or
    /// [`Self::verify_endpoint`] says otherwise.
    pub fn payload(&self) -> postcard::Result<T> {
        postcard::from_bytes(&self.payload_bytes)
    }

    /// Bytes the encoded payload takes, without decoding it.
    #[must_use]
    pub const fn payload_len(&self) -> usize {
        self.payload_bytes.len()
    }

    /// Checks the signature against `did`'s authentication keys.
    pub async fn verify_did(&self, did: &Did, resolver: &Resolver) -> Result<(), VerifyError>
    where
        T: Sync,
    {
        let doc = resolver
            .resolve(did)
            .await
            .map_err(|source| VerifyError::Unresolvable {
                did: did.clone(),
                source,
            })?;
        self.verify_document(did, &doc)
    }

    /// Checks the signature against the authentication keys of `doc`, which is
    /// `did`'s already-resolved document.
    pub fn verify_document(&self, did: &Did, doc: &Document) -> Result<(), VerifyError> {
        let methods = doc
            .authentication
            .as_ref()
            .ok_or_else(|| VerifyError::NoAuthenticationKey(did.clone()))?;

        let signing_bytes = self.signing_bytes();

        for method in methods {
            if let Some(map) = doc.resolve_verification_method(method)
                && let Some(key) = &map.public_key_jwk
                && jwk::verify(key, &self.signature, &signing_bytes).is_ok()
            {
                return Ok(());
            }
        }

        Err(VerifyError::NotSigned(did.clone()))
    }

    /// Checks the signature against an endpoint key, as made by
    /// [`EndpointSigner`].
    pub fn verify_endpoint(&self, endpoint: EndpointId) -> Result<(), VerifyError> {
        let signature = <[u8; 64]>::try_from(self.signature.as_slice())
            .map_err(|_| VerifyError::NotSignedByEndpoint(endpoint))?;
        endpoint
            .verify(&self.signing_bytes(), &Signature::from_bytes(&signature))
            .map_err(|_| VerifyError::NotSignedByEndpoint(endpoint))
    }

    /// Checks the signature against the namespace key `ns` names, as made by
    /// [`NamespaceSigner`]. Needs no resolver: the id is the public key.
    pub fn verify_namespace(&self, ns: NamespaceId) -> Result<(), VerifyError> {
        let forged = || VerifyError::NotSignedByNamespace(ns);
        let signature = <[u8; 64]>::try_from(self.signature.as_slice()).map_err(|_| forged())?;
        let key = NamespacePublicKey::from_bytes(ns.as_bytes()).map_err(|_| forged())?;
        key.verify(&self.signing_bytes(), &Signature::from_bytes(&signature))
            .map_err(|_| forged())
    }
}

/// Contexts are ASCII literals, so a NUL terminator keeps the framing
/// unambiguous without a length prefix.
fn signing_bytes(context: &str, payload: &[u8]) -> Vec<u8> {
    debug_assert!(!context.contains('\0'), "context must not contain NUL");
    let mut out = Vec::with_capacity(context.len() + 1 + payload.len());
    out.extend_from_slice(context.as_bytes());
    out.push(0);
    out.extend_from_slice(payload);
    out
}

pub trait Signable
where
    for<'a> Self: Serialize + Deserialize<'a>,
{
    /// Distinct payload types can encode to identical bytes, so without this a
    /// signature collected from one protocol would verify as another. Never
    /// reuse a context.
    const SIGNING_CONTEXT: &'static str;

    fn sign(&self, key: &impl Signer) -> anyhow::Result<Signed<Self>> {
        Signed::sign(self, key)
    }
}

/// Signs with an endpoint's key, for payloads verified by
/// [`Signed::verify_endpoint`].
pub struct EndpointSigner<'a>(pub &'a iroh::SecretKey);

impl Signer for EndpointSigner<'_> {
    type Error = Infallible;

    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Infallible> {
        Ok(self.0.sign(message).to_bytes().to_vec())
    }
}

/// Signs with a namespace's write key, for payloads verified by
/// [`Signed::verify_namespace`].
pub struct NamespaceSigner<'a>(pub &'a NamespaceSecret);

impl Signer for NamespaceSigner<'_> {
    type Error = Infallible;

    fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Infallible> {
        Ok(self.0.sign(message).to_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use iroh::SecretKey;

    use super::*;

    #[derive(Serialize, Deserialize)]
    struct A(u8);
    impl Signable for A {
        const SIGNING_CONTEXT: &'static str = "test/a";
    }

    #[derive(Serialize, Deserialize)]
    struct B(u8);
    impl Signable for B {
        const SIGNING_CONTEXT: &'static str = "test/b";
    }

    impl<T: Signable> Signed<T> {
        fn from_payload(payload_bytes: Vec<u8>) -> Self {
            Self {
                payload_bytes,
                signature: Vec::new(),
                _type: PhantomData,
            }
        }
    }

    #[test]
    fn identical_payloads_sign_over_different_bytes() {
        let a = Signed::<A>::from_payload(postcard::to_stdvec(&A(7)).expect("encode a"));
        let b = Signed::<B>::from_payload(postcard::to_stdvec(&B(7)).expect("encode b"));

        assert_eq!(a.payload_bytes, b.payload_bytes);
        assert_ne!(
            a.signing_bytes(),
            b.signing_bytes(),
            "a signature from one protocol must not verify as another"
        );
    }

    #[test]
    fn context_cannot_absorb_payload_prefix() {
        let split = Signed::<A>::from_payload(b"\0extra".to_vec());
        let joined = Signed::<A>::from_payload(b"extra".to_vec());

        assert_ne!(split.signing_bytes(), joined.signing_bytes());
    }

    #[test]
    fn an_endpoint_signature_verifies_only_for_its_endpoint() {
        let key = SecretKey::generate();
        let signed = A(7).sign(&EndpointSigner(&key)).expect("sign");

        signed
            .verify_endpoint(key.public())
            .expect("the signing endpoint verifies");
        assert!(
            signed
                .verify_endpoint(SecretKey::generate().public())
                .is_err()
        );
    }

    #[test]
    fn a_namespace_signature_verifies_only_for_its_namespace() {
        let ns = NamespaceSecret::from_bytes(&[1; 32]);
        let signed = A(7).sign(&NamespaceSigner(&ns)).expect("sign");

        signed
            .verify_namespace(ns.id())
            .expect("the signing namespace verifies");
        assert!(
            signed
                .verify_namespace(NamespaceSecret::from_bytes(&[2; 32]).id())
                .is_err()
        );
    }
}
