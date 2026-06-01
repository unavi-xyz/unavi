use std::fmt::Display;

use serde::{
    Deserialize,
    Serialize,
};

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

impl<const N: usize> Default for ByteArray<N> {
    fn default() -> Self {
        Self([0; N])
    }
}

impl<const N: usize> Display for ByteArray<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl<const N: usize> From<[u8; N]> for ByteArray<N> {
    fn from(bytes: [u8; N]) -> Self {
        Self(bytes)
    }
}

impl<const N: usize> Serialize for ByteArray<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, const N: usize> Deserialize<'de> for ByteArray<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ByteArrayVisitor<const N: usize>;

        impl<'de, const N: usize> serde::de::Visitor<'de> for ByteArrayVisitor<N> {
            type Value = ByteArray<N>;

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
                Ok(ByteArray(arr))
            }

            fn visit_bytes<E: serde::de::Error>(self, b: &[u8]) -> Result<Self::Value, E> {
                let arr: [u8; N] = b
                    .try_into()
                    .map_err(|_| E::invalid_length(b.len(), &self))?;
                Ok(ByteArray(arr))
            }

            fn visit_byte_buf<E: serde::de::Error>(self, b: Vec<u8>) -> Result<Self::Value, E> {
                self.visit_bytes(&b)
            }
        }

        deserializer.deserialize_tuple(N, ByteArrayVisitor::<N>)
    }
}

impl From<blake3::Hash> for ByteArray<32> {
    fn from(value: blake3::Hash) -> Self {
        Self(value.into())
    }
}

impl From<ByteArray<32>> for blake3::Hash {
    fn from(value: ByteArray<32>) -> Self {
        Self::from_bytes(value.0)
    }
}

impl From<&ByteArray<32>> for blake3::Hash {
    fn from(value: &ByteArray<32>) -> Self {
        Self::from_bytes(value.0)
    }
}

#[cfg(feature = "loro")]
mod loro_impls {
    use loro_surgeon::{
        Hydrate,
        Reconcile,
        error::{
            HydrateError,
            ReconcileError,
        },
        reconcile::{
            NoKey,
            Reconciler,
        },
    };

    use super::ByteArray;

    impl<const N: usize> Hydrate for ByteArray<N> {
        fn hydrate_binary(b: &[u8]) -> Result<Self, HydrateError> {
            let arr: [u8; N] = b.try_into().map_err(|_| HydrateError::Unexpected {
                expected: "binary of correct length",
                found:    "binary of wrong length",
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
}
