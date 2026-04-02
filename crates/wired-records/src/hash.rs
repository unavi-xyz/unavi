use std::fmt;

use serde::{Deserialize, Serialize};

/// 32-byte BLAKE3 hash, serializable without the blake3 crate dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HydratedHash(pub [u8; 32]);

impl fmt::Display for HydratedHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

#[cfg(feature = "loro")]
mod blake3_impls {
    use super::*;

    impl From<blake3::Hash> for HydratedHash {
        fn from(h: blake3::Hash) -> Self {
            Self(*h.as_bytes())
        }
    }

    impl From<HydratedHash> for blake3::Hash {
        fn from(h: HydratedHash) -> Self {
            blake3::Hash::from_bytes(h.0)
        }
    }
}

#[cfg(feature = "loro")]
mod loro_impls {
    use loro::LoroValue;
    use loro_surgeon::{Hydrate, HydrateError, Reconcile, ReconcileError, loro::LoroMap};

    use super::*;

    impl Hydrate for HydratedHash {
        fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
            let LoroValue::Binary(bytes) = value else {
                return Err(HydrateError::TypeMismatch {
                    expected: "binary".into(),
                    actual: format!("{value:?}").into(),
                });
            };
            let arr: [u8; 32] = bytes[..]
                .try_into()
                .map_err(|_| HydrateError::Custom("expected 32 bytes for hash".into()))?;
            Ok(Self(arr))
        }
    }

    impl Reconcile for HydratedHash {
        fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
            Err(ReconcileError::Custom(
                "HydratedHash cannot be reconciled as a root container".into(),
            ))
        }

        fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
            map.insert(key, self.0.to_vec())?;
            Ok(())
        }

        fn to_loro_value(&self) -> Option<LoroValue> {
            Some(LoroValue::Binary(self.0.to_vec().into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use loro::{LoroDoc, LoroValue};

    use super::*;

    #[test]
    fn roundtrip() {
        use loro_surgeon::{Hydrate, Reconcile};

        let doc = LoroDoc::new();
        let map = doc.get_map("test");

        let hash = HydratedHash(*blake3::hash(b"test data").as_bytes());
        hash.reconcile_field(&map, "hash").expect("reconcile");

        let value = map.get_deep_value();
        let LoroValue::Map(m) = &value else {
            panic!("expected map");
        };
        let loaded = HydratedHash::hydrate(m.get("hash").expect("missing")).expect("hydrate");
        assert_eq!(hash, loaded);
    }
}
