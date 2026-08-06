use std::marker::PhantomData;

use serde::{
    Deserialize,
    Serialize,
};
use xdid::{
    core::did::Did,
    methods::key::keys::Signer,
};

use crate::{
    auth::jwk::verify_jwk_signature,
    resolve::resolve,
};

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

    /// The exact bytes a signature covers: the payload under
    /// [`Signable::SIGNING_CONTEXT`]. Verification must use this, never the
    /// payload alone.
    #[must_use]
    pub fn signing_bytes(&self) -> Vec<u8> {
        signing_bytes(T::SIGNING_CONTEXT, &self.payload_bytes)
    }

    pub fn payload(&self) -> postcard::Result<T> {
        postcard::from_bytes(&self.payload_bytes)
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
    /// Binds a signature to the protocol that asked for it.
    ///
    /// Distinct payload types routinely encode to identical bytes —
    /// `Challenge` and `Presence` are both a DID, two 32-byte ids and a
    /// timestamp — so without this a signature collected from one protocol
    /// would verify as another. Give every type its own string; never reuse
    /// one.
    const SIGNING_CONTEXT: &'static str;

    fn sign(&self, key: &impl Signer) -> anyhow::Result<SignedBytes<Self>> {
        SignedBytes::sign(self, key)
    }
}

/// Verifies that `signed` was produced by an authentication key of `did`, by
/// resolving the DID document and checking the signature against its
/// authentication verification methods.
pub async fn verify_did_signature<T>(signed: &SignedBytes<T>, did: &Did) -> bool
where
    T: Signable + Sync,
{
    let Some(doc) = resolve(did).await else {
        return false;
    };
    let Some(auth_methods) = &doc.authentication else {
        return false;
    };
    let signing_bytes = signed.signing_bytes();
    for method in auth_methods {
        if let Some(map) = doc.resolve_verification_method(method)
            && let Some(jwk) = &map.public_key_jwk
            && verify_jwk_signature(jwk, signed.signature(), &signing_bytes)
        {
            return true;
        }
    }
    false
}

pub struct IrohSigner<'a>(pub &'a iroh::SecretKey);

impl Signer for IrohSigner<'_> {
    fn sign(&self, message: &[u8]) -> anyhow::Result<Vec<u8>> {
        let sig = self.0.sign(message);
        Ok(sig.to_bytes().to_vec())
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

    #[test]
    fn identical_payloads_sign_over_different_bytes() {
        let a = SignedBytes::<A>::from_payload(postcard::to_stdvec(&A(7)).expect("encode a"));
        let b = SignedBytes::<B>::from_payload(postcard::to_stdvec(&B(7)).expect("encode b"));

        assert_eq!(a.payload_bytes, b.payload_bytes);
        assert_ne!(a.signing_bytes(), b.signing_bytes());
    }

    #[test]
    fn context_cannot_absorb_payload_prefix() {
        let split = SignedBytes::<A>::from_payload(b"\0extra".to_vec());
        let joined = SignedBytes::<A>::from_payload(b"extra".to_vec());

        assert_ne!(split.signing_bytes(), joined.signing_bytes());
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
}
