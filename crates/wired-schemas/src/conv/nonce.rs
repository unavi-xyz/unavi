use loro::LoroValue;
use loro_surgeon::{HydrateError, ReconcileError, loro::LoroMap};

use crate::record::RecordNonce;

/// # Errors
///
/// Returns an error if the value is not binary or has invalid length.
pub fn hydrate(value: &LoroValue) -> Result<RecordNonce, HydrateError> {
    let LoroValue::Binary(bytes) = value else {
        return Err(HydrateError::TypeMismatch {
            path: String::new(),
            expected: "binary".into(),
            actual: format!("{value:?}"),
        });
    };
    bytes
        .to_vec()
        .try_into()
        .map_err(|_| HydrateError::Custom("invalid nonce length".into()))
}

/// # Errors
///
/// Returns an error if the Loro operation fails.
pub fn reconcile(nonce: &RecordNonce, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
    map.insert(key, nonce.as_slice())?;
    Ok(())
}
