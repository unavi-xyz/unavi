use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HydratedHash(pub blake3::Hash);

impl fmt::Display for HydratedHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)?;
        Ok(())
    }
}

impl From<blake3::Hash> for HydratedHash {
    fn from(h: blake3::Hash) -> Self {
        Self(h)
    }
}

impl From<HydratedHash> for blake3::Hash {
    fn from(h: HydratedHash) -> Self {
        h.0
    }
}

#[cfg(feature = "loro")]
mod loro_impls {
    use loro::LoroValue;
    use loro_surgeon::{Hydrate, HydrateError, Reconcile, ReconcileError, loro::LoroMap};

    use super::HydratedHash;

    impl Hydrate for HydratedHash {
        fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
            let LoroValue::Binary(binary) = value else {
                return Err(HydrateError::TypeMismatch {
                    expected: "binary".into(),
                    actual: format!("{value:?}").into(),
                });
            };
            let bytes = binary[..]
                .try_into()
                .map_err(|_| HydrateError::Custom("expected 32 bytes for hash".into()))?;
            Ok(Self(blake3::Hash::from_bytes(bytes)))
        }
    }

    impl Reconcile for HydratedHash {
        fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
            Err(ReconcileError::Custom(
                "HydratedHash cannot be reconciled as a root container".into(),
            ))
        }

        fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
            map.insert(key, self.0.as_bytes().to_vec())?;
            Ok(())
        }

        fn to_loro_value(&self) -> Option<LoroValue> {
            Some(LoroValue::Binary(self.0.as_bytes().to_vec().into()))
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

        let hash = HydratedHash(blake3::hash(b"test data"));
        hash.reconcile_field(&map, "hash").expect("reconcile");

        let value = map.get_deep_value();
        let LoroValue::Map(m) = &value else {
            panic!("expected map");
        };
        let loaded = HydratedHash::hydrate(m.get("hash").expect("missing")).expect("hydrate");
        assert_eq!(hash, loaded);
    }
}
