use std::marker::PhantomData;

use serde::{
    Deserialize,
    Serialize,
};
use xdid::{
    core::did::Did,
    methods::key::Signer,
    resolver::DidResolver,
};

use crate::auth::jwk::verify_jwk_signature;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedBytes<T>
where
    for<'a> T: Serialize + Deserialize<'a>,
{
    payload_bytes: Vec<u8>,
    signature:     Vec<u8>,
    _type:         PhantomData<T>,
}

impl<T> SignedBytes<T>
where
    for<'a> T: Serialize + Deserialize<'a>,
{
    fn sign(payload: &T, key: &impl Signer) -> anyhow::Result<Self> {
        let payload_bytes = postcard::to_stdvec(payload)?;
        let signature = key.sign(&payload_bytes)?;

        Ok(Self {
            payload_bytes,
            signature,
            _type: PhantomData,
        })
    }

    /// Reconstructs a `SignedBytes` from stored components.
    #[must_use]
    pub const fn from_parts(payload_bytes: Vec<u8>, signature: Vec<u8>) -> Self {
        Self {
            payload_bytes,
            signature,
            _type: PhantomData,
        }
    }

    #[must_use]
    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    #[must_use]
    pub fn payload_bytes(&self) -> &[u8] {
        &self.payload_bytes
    }

    pub fn payload(&self) -> postcard::Result<T> {
        postcard::from_bytes(&self.payload_bytes)
    }
}

pub trait Signable
where
    for<'a> Self: Serialize + Deserialize<'a>,
{
    fn sign(&self, key: &impl Signer) -> anyhow::Result<SignedBytes<Self>> {
        SignedBytes::sign(self, key)
    }
}

/// Verifies that `signed` was produced by an authentication key of `did`, by
/// resolving the DID document and checking the signature against its
/// authentication verification methods.
pub async fn verify_did_signature<T>(signed: &SignedBytes<T>, did: &Did) -> bool
where
    for<'a> T: Serialize + Deserialize<'a> + Sync,
{
    let Ok(resolver) = DidResolver::new() else {
        return false;
    };
    let Ok(doc) = resolver.resolve(did).await else {
        return false;
    };
    let Some(auth_methods) = &doc.authentication else {
        return false;
    };
    for method in auth_methods {
        if let Some(map) = doc.resolve_verification_method(method)
            && let Some(jwk) = &map.public_key_jwk
            && verify_jwk_signature(jwk, signed.signature(), signed.payload_bytes())
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
