use std::marker::PhantomData;

use serde::{
    Deserialize,
    Serialize,
};
use xdid::{
    core::{
        ResolutionError,
        did::Did,
    },
    methods::key::keys::Signer,
};

use crate::{
    jwk,
    resolve::Resolver,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedBytes<T>
where
    T: Signable,
{
    payload_bytes: Vec<u8>,
    signature:     Vec<u8>,
    _type:         PhantomData<T>,
}

impl<T> SignedBytes<T>
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

    #[must_use]
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    /// The exact bytes a signature covers. The payload alone is not signed.
    #[must_use]
    pub fn signing_bytes(&self) -> Vec<u8> {
        signing_bytes(T::SIGNING_CONTEXT, &self.payload_bytes)
    }

    pub fn payload(&self) -> postcard::Result<T> {
        postcard::from_bytes(&self.payload_bytes)
    }

    /// Checks the signature against `did`'s authentication keys.
    pub async fn verify(&self, did: &Did, resolver: &Resolver) -> Result<(), VerifyError>
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

    fn sign(&self, key: &impl Signer) -> anyhow::Result<SignedBytes<Self>> {
        SignedBytes::sign(self, key)
    }
}

pub struct IrohSigner<'a>(pub &'a iroh::SecretKey);

impl Signer for IrohSigner<'_> {
    fn sign(&self, message: &[u8]) -> anyhow::Result<Vec<u8>> {
        Ok(self.0.sign(message).to_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
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

    impl<T: Signable> SignedBytes<T> {
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
        let a = SignedBytes::<A>::from_payload(postcard::to_stdvec(&A(7)).expect("encode a"));
        let b = SignedBytes::<B>::from_payload(postcard::to_stdvec(&B(7)).expect("encode b"));

        assert_eq!(a.payload_bytes, b.payload_bytes);
        assert_ne!(
            a.signing_bytes(),
            b.signing_bytes(),
            "a signature from one protocol must not verify as another"
        );
    }

    #[test]
    fn context_cannot_absorb_payload_prefix() {
        let split = SignedBytes::<A>::from_payload(b"\0extra".to_vec());
        let joined = SignedBytes::<A>::from_payload(b"extra".to_vec());

        assert_ne!(split.signing_bytes(), joined.signing_bytes());
    }
}
