use std::fmt;

use serde::{Deserialize, Serialize};

/// Endpoint identity bytes — a 32-byte public key identifying a network node.
///
/// Wraps `iroh::EndpointId` bytes without requiring the iroh crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HydratedEndpoint(pub [u8; 32]);

impl fmt::Display for HydratedEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

#[cfg(feature = "loro")]
mod loro_impls {
    use loro::LoroValue;
    use loro_surgeon::{Hydrate, HydrateError, Reconcile, ReconcileError, loro::LoroMap};

    use super::HydratedEndpoint;

    impl Hydrate for HydratedEndpoint {
        fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
            let LoroValue::Binary(bytes) = value else {
                return Err(HydrateError::TypeMismatch {
                    expected: "binary".into(),
                    actual: format!("{value:?}").into(),
                });
            };
            let arr: [u8; 32] = bytes[..]
                .try_into()
                .map_err(|_| HydrateError::Custom("expected 32 bytes for endpoint".into()))?;
            Ok(Self(arr))
        }
    }

    impl Reconcile for HydratedEndpoint {
        fn reconcile(&self, _map: &LoroMap) -> Result<(), ReconcileError> {
            Err(ReconcileError::Custom(
                "HydratedEndpoint cannot be reconciled as a root container".into(),
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
