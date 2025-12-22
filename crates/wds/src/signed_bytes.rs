use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use xdid::methods::key::Signer;

#[derive(Debug, Serialize, Deserialize)]
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
    /// # Errors
    ///
    /// Errors if the payload could not be serialized, or the bytes could not
    /// be signed.
    pub fn sign(payload: &T, key: &impl Signer) -> anyhow::Result<Self> {
        let payload_bytes = postcard::to_stdvec(payload)?;
        let signature = key.sign(&payload_bytes)?;

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
