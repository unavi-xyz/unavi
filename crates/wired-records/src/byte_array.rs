use std::fmt::Display;

use loro_surgeon::{
    Hydrate,
    Reconcile,
};
use serde::{
    Deserialize,
    Serialize,
};

#[derive(Hydrate, Reconcile, Debug, Clone)]
pub struct ByteArray<const N: usize>(pub loro_surgeon::bytes::ByteArray<N>);

impl<const N: usize> ByteArray<N> {
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; N] {
        self.0.as_bytes()
    }
}

impl<const N: usize> Default for ByteArray<N> {
    fn default() -> Self {
        Self(loro_surgeon::bytes::ByteArray::new([0; N]))
    }
}

impl<const N: usize> Display for ByteArray<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.0.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl<const N: usize> Serialize for ByteArray<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.0.serialize(serializer)
    }
}

impl<'de, const N: usize> Deserialize<'de> for ByteArray<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let vec = <Vec<u8>>::deserialize(deserializer)?;
        let arr: [u8; N] = vec
            .try_into()
            .map_err(|_| serde::de::Error::custom(format!("expected {N} bytes")))?;
        Ok(Self(loro_surgeon::bytes::ByteArray(arr)))
    }
}

impl From<blake3::Hash> for ByteArray<32> {
    fn from(value: blake3::Hash) -> Self {
        Self(loro_surgeon::bytes::ByteArray(value.into()))
    }
}

impl From<ByteArray<32>> for blake3::Hash {
    fn from(value: ByteArray<32>) -> Self {
        Self::from_bytes(value.0.0)
    }
}

impl From<&ByteArray<32>> for blake3::Hash {
    fn from(value: &ByteArray<32>) -> Self {
        Self::from_bytes(value.0.0)
    }
}
