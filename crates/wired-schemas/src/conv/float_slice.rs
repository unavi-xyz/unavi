use loro::{LoroList, LoroValue};
use loro_surgeon::{HydrateError, ReconcileError, loro::LoroMap};
use smol_str::SmolStr;

/// # Errors
///
/// Returns an error if the value is not a float list or has invalid length.
pub fn hydrate<T: TryFrom<Vec<f64>>>(value: &LoroValue) -> Result<T, HydrateError> {
    let LoroValue::List(list) = value else {
        return Err(HydrateError::TypeMismatch {
            path: SmolStr::default(),
            expected: "list".into(),
            actual: format!("{value:?}").into(),
        });
    };
    list.iter()
        .map(|v| {
            v.as_double()
                .copied()
                .ok_or_else(|| HydrateError::TypeMismatch {
                    path: "[list item]".into(),
                    expected: "f64".into(),
                    actual: format!("{v:?}").into(),
                })
        })
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| HydrateError::Custom("invalid slice length".into()))
}

/// # Errors
///
/// Returns an error if the Loro operation fails.
pub fn reconcile(slice: &[f64], map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
    let list = LoroList::new();
    for x in slice {
        list.push(*x)?;
    }
    map.insert_container(key, list)?;
    Ok(())
}

pub mod optional {
    use loro::LoroValue;
    use loro_surgeon::HydrateError;

    /// # Errors
    ///
    /// Returns an error if the value is the wrong type.
    pub fn hydrate<T: TryFrom<Vec<f64>>>(value: &LoroValue) -> Result<Option<T>, HydrateError> {
        if value.is_null() {
            return Ok(None);
        }

        super::hydrate(value).map(Some)
    }
}
