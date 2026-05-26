//! Newtypes that map to [`LoroValue::Binary`].
//!
//! `Vec<u8>` and `[u8; N]` go through the regular list paths; binary fields
//! must use these explicit wrappers.

use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    error::{HydrateError, ReconcileError},
    hydrate::Hydrate,
    reconcile::{NoKey, Reconcile, Reconciler},
};

/// Cap to defend against malicious or malformed documents claiming
/// gigantic binary payloads. 256 MiB is well above any legitimate
/// in-document blob we expect.
pub const MAX_BYTES_LEN: usize = 256 * 1024 * 1024;

/// A variable-length byte buffer that round-trips through `LoroValue::Binary`.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct Bytes(pub Vec<u8>);

impl Bytes {
    #[must_use]
    pub const fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    #[must_use]
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl From<Bytes> for Vec<u8> {
    fn from(b: Bytes) -> Self {
        b.0
    }
}

impl Deref for Bytes {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Bytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Hydrate for Bytes {
    fn hydrate_binary(b: &[u8]) -> Result<Self, HydrateError> {
        if b.len() > MAX_BYTES_LEN {
            return Err(HydrateError::Unexpected {
                expected: "binary payload within size limit",
                found: "binary payload exceeds MAX_BYTES_LEN",
            });
        }
        Ok(Self(b.to_vec()))
    }
}

impl Reconcile for Bytes {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        r.bytes(&self.0)
    }
}

impl Serialize for Bytes {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        serde::Serialize::serialize(&self.0, s)
    }
}

impl<'de> Deserialize<'de> for Bytes {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Vec::<u8>::deserialize(d).map(Self)
    }
}

/// A fixed-size byte array that round-trips through `LoroValue::Binary`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ByteArray<const N: usize>(pub [u8; N]);

impl<const N: usize> ByteArray<N> {
    #[must_use]
    pub const fn new(bytes: [u8; N]) -> Self {
        Self(bytes)
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; N] {
        &self.0
    }
}

impl<const N: usize> From<[u8; N]> for ByteArray<N> {
    fn from(bytes: [u8; N]) -> Self {
        Self(bytes)
    }
}

impl<const N: usize> Default for ByteArray<N> {
    fn default() -> Self {
        Self([0u8; N])
    }
}

impl<const N: usize> Hydrate for ByteArray<N> {
    fn hydrate_binary(b: &[u8]) -> Result<Self, HydrateError> {
        let arr: [u8; N] = b.try_into().map_err(|_| HydrateError::Unexpected {
            expected: "binary of correct length",
            found: "binary of wrong length",
        })?;
        Ok(Self(arr))
    }
}

impl<const N: usize> Reconcile for ByteArray<N> {
    type Key = NoKey;
    fn reconcile<R: Reconciler>(&self, r: R) -> Result<(), ReconcileError> {
        r.bytes(&self.0)
    }
}

impl<const N: usize> Serialize for ByteArray<N> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(s)
    }
}

impl<'de, const N: usize> Deserialize<'de> for ByteArray<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V<const N: usize>;
        impl<'de, const N: usize> serde::de::Visitor<'de> for V<N> {
            type Value = [u8; N];
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "byte array of length {N}")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut seq: A,
            ) -> Result<Self::Value, A::Error> {
                let mut arr = [0u8; N];
                for (i, slot) in arr.iter_mut().enumerate() {
                    *slot = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
                }
                Ok(arr)
            }
            fn visit_bytes<E: serde::de::Error>(self, b: &[u8]) -> Result<Self::Value, E> {
                b.try_into()
                    .map_err(|_| serde::de::Error::invalid_length(b.len(), &self))
            }
        }
        d.deserialize_seq(V::<N>).map(Self)
    }
}
