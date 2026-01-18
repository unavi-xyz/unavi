use loro::LoroValue;
use loro_surgeon::{HydrateError, ReconcileError, loro::LoroMap};

/// # Errors
///
/// Returns an error if the value is not binary or has invalid length.
pub fn hydrate<T: TryFrom<Vec<u8>>>(value: &LoroValue) -> Result<T, HydrateError> {
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
        .map_err(|_| HydrateError::Custom("invalid slice length".into()))
}

/// # Errors
///
/// Returns an error if the Loro operation fails.
pub fn reconcile(slice: &[u8], map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
    map.insert(key, slice)?;
    Ok(())
}
