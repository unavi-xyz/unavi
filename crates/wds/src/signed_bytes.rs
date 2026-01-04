use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use xdid::methods::key::Signer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedBytes<T>
where
    for<'a> T: Serialize + Deserialize<'a>,
{
    payload_bytes: Vec<u8>,
    signature: Vec<u8>,
    _type: PhantomData<T>,
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

    /// # Errors
    ///
    /// Errors if the payload could not be deserialized.
    pub fn payload(&self) -> postcard::Result<T> {
        postcard::from_bytes(&self.payload_bytes)
    }
}

pub trait Signable
where
    for<'a> Self: Serialize + Deserialize<'a>,
{
    /// # Errors
    ///
    /// Errors if the payload could not be serialized, or the bytes could not
    /// be signed.
    fn sign(&self, key: &impl Signer) -> anyhow::Result<SignedBytes<Self>> {
        SignedBytes::sign(self, key)
    }
}
