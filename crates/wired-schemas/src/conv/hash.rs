use std::collections::BTreeMap;

use blake3::Hash;
use loro::LoroValue;
use loro_surgeon::{HydrateError, ReconcileError, loro::LoroMap};
use smol_str::SmolStr;

/// Hydrate a [`Hash`] from a Loro string value (hex encoded).
///
/// # Errors
///
/// Returns an error if the value is not a string or cannot be parsed as a hash.
pub fn hydrate(value: &LoroValue) -> Result<Hash, HydrateError> {
    let LoroValue::String(s) = value else {
        return Err(HydrateError::TypeMismatch {
            path: String::new(),
            expected: "string".into(),
            actual: format!("{value:?}"),
        });
    };
    Hash::from_hex(s.as_ref()).map_err(|e| HydrateError::Custom(e.to_string()))
}

/// # Errors
///
/// Returns an error if the Loro operation fails.
pub fn reconcile(hash: &Hash, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
    map.insert(key, hash.to_string())?;
    Ok(())
}

/// Module for handling `BTreeMap<SmolStr, Hash>` fields (schema maps).
pub mod map {
    use super::{BTreeMap, Hash, HydrateError, LoroMap, LoroValue, ReconcileError, SmolStr};

    /// Hydrate a `BTreeMap<SmolStr, Hash>` from a Loro map value.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a map or contains invalid hashes.
    pub fn hydrate(value: &LoroValue) -> Result<BTreeMap<SmolStr, Hash>, HydrateError> {
        let LoroValue::Map(map) = value else {
            return Err(HydrateError::TypeMismatch {
                path: String::new(),
                expected: "map".into(),
                actual: format!("{value:?}"),
            });
        };

        map.iter()
            .map(|(k, v)| {
                let hash = super::hydrate(v)?;
                Ok((k.into(), hash))
            })
            .collect()
    }

    /// # Errors
    ///
    /// Returns an error if the Loro operation fails.
    pub fn reconcile(
        schemas: &BTreeMap<SmolStr, Hash>,
        map: &LoroMap,
        key: &str,
    ) -> Result<(), ReconcileError> {
        let nested = map.get_or_create_container(key, loro::LoroMap::new())?;
        for (k, v) in schemas {
            nested.insert(k.as_str(), v.to_string())?;
        }
        Ok(())
    }
}
