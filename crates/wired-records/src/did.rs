use std::fmt;

use serde::{Deserialize, Serialize};

/// A DID string, serializable without the xdid crate dependency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct HydratedDid(pub String);

impl fmt::Display for HydratedDid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "loro")]
mod xdid_impls {
    use xdid::core::did::Did;

    use super::*;

    impl From<Did> for HydratedDid {
        fn from(d: Did) -> Self {
            Self(d.to_string())
        }
    }
}

#[cfg(feature = "loro")]
mod loro_impls {
    use loro::LoroValue;
    use loro_surgeon::{Hydrate, HydrateError, Reconcile, ReconcileError, loro::LoroMap};

    use super::*;

    impl Hydrate for HydratedDid {
        fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
            let LoroValue::String(s) = value else {
                return Err(HydrateError::TypeMismatch {
                    expected: "string".into(),
                    actual: format!("{value:?}").into(),
                });
            };
            Ok(Self(s.to_string()))
        }
    }

    impl Reconcile for HydratedDid {
        fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
            Err(ReconcileError::Custom(
                "HydratedDid cannot be reconciled as a root container".into(),
            ))
        }

        fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
            map.insert(key, self.0.as_str())?;
            Ok(())
        }

        fn to_loro_value(&self) -> Option<LoroValue> {
            Some(LoroValue::String(self.0.clone().into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use loro::{LoroDoc, LoroValue};
    use loro_surgeon::{Hydrate, Reconcile};

    use super::*;

    #[test]
    fn roundtrip() {
        let doc = LoroDoc::new();
        let map = doc.get_map("test");

        let wdid = HydratedDid("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".into());
        wdid.reconcile_field(&map, "did").expect("reconcile");

        let value = map.get_deep_value();
        let LoroValue::Map(m) = &value else {
            panic!("expected map");
        };
        let loaded = HydratedDid::hydrate(m.get("did").expect("missing")).expect("hydrate");
        assert_eq!(wdid, loaded);
    }
}
