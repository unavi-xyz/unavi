use std::collections::BTreeMap;

use blake3::Hash;
use loro::LoroValue;
use loro_surgeon::{HydrateError, ReconcileError, loro::LoroMap};
use smol_str::SmolStr;

/// Hydrate a [`Hash`] from a Loro binary value (32 bytes).
///
/// # Errors
///
/// Returns an error if the value is not binary or is not 32 bytes.
pub fn hydrate(value: &LoroValue) -> Result<Hash, HydrateError> {
    let LoroValue::Binary(bytes) = value else {
        return Err(HydrateError::TypeMismatch {
            path: SmolStr::default(),
            expected: "binary".into(),
            actual: format!("{value:?}").into(),
        });
    };
    let slice: &[u8] = bytes.as_ref();
    let arr: [u8; 32] = slice
        .try_into()
        .map_err(|_| HydrateError::Custom("expected 32 bytes for hash".into()))?;
    Ok(Hash::from_bytes(arr))
}

/// # Errors
///
/// Returns an error if the Loro operation fails.
pub fn reconcile(hash: &Hash, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
    map.insert(key, hash.as_bytes().to_vec())?;
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
                path: SmolStr::default(),
                expected: "map".into(),
                actual: format!("{value:?}").into(),
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
            nested.insert(k.as_str(), v.as_bytes().to_vec())?;
        }
        Ok(())
    }
}

pub mod optional {
    use blake3::Hash;
    use loro::LoroValue;
    use loro_surgeon::HydrateError;

    /// # Errors
    ///
    /// Returns an error if the value is the wrong type.
    pub fn hydrate(value: &LoroValue) -> Result<Option<Hash>, HydrateError> {
        if value.is_null() {
            return Ok(None);
        }

        super::hydrate(value).map(Some)
    }
}
