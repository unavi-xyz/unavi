//! Converter for [`Did`] types.

use std::str::FromStr;

use loro::LoroValue;
use loro_surgeon::{HydrateError, ReconcileError, loro::LoroMap};
use xdid::core::did::Did;

/// Hydrate a [`Did`] from a Loro string value.
///
/// # Errors
///
/// Returns an error if the value is not a string or cannot be parsed as a DID.
pub fn hydrate(value: &LoroValue) -> Result<Did, HydrateError> {
    let LoroValue::String(s) = value else {
        return Err(HydrateError::TypeMismatch {
            path: String::new(),
            expected: "string".into(),
            actual: format!("{value:?}"),
        });
    };
    Did::from_str(s).map_err(|e| HydrateError::Custom(e.to_string()))
}

/// Reconcile a [`Did`] as a Loro string value.
///
/// # Errors
///
/// Returns an error if the Loro operation fails.
pub fn reconcile_field(did: &Did, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
    map.insert(key, did.to_string())?;
    Ok(())
}

/// Module for handling `Vec<Did>` fields.
pub mod list {
    use super::{Did, HydrateError, LoroMap, LoroValue, ReconcileError};

    /// Hydrate a `Vec<Did>` from a Loro list value.
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a list or contains invalid DIDs.
    pub fn hydrate(value: &LoroValue) -> Result<Vec<Did>, HydrateError> {
        let LoroValue::List(list) = value else {
            return Err(HydrateError::TypeMismatch {
                path: String::new(),
                expected: "list".into(),
                actual: format!("{value:?}"),
            });
        };

        list.iter().map(super::hydrate).collect()
    }

    /// Reconcile a `Vec<Did>` as a Loro list value.
    ///
    /// # Errors
    ///
    /// Returns an error if the Loro operation fails.
    pub fn reconcile_field(dids: &[Did], map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        let strings: Vec<String> = dids.iter().map(Did::to_string).collect();
        map.insert(key, strings)?;
        Ok(())
    }
}
