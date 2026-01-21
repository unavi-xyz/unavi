use loro::LoroValue;
use loro_surgeon::{Hydrate, HydrateError};
use smol_str::SmolStr;

/// Hydrate a tree by extracting `meta` from each node.
///
/// # Errors
///
/// Returns an error if the value is not a list of node maps.
pub fn hydrate<T: Hydrate>(value: &LoroValue) -> Result<Vec<T>, HydrateError> {
    let LoroValue::List(nodes) = value else {
        return Err(HydrateError::TypeMismatch {
            path: SmolStr::default(),
            expected: "list".into(),
            actual: format!("{value:?}").into(),
        });
    };

    nodes
        .iter()
        .filter_map(|node| {
            let LoroValue::Map(node_map) = node else {
                return Some(Err(HydrateError::TypeMismatch {
                    path: SmolStr::default(),
                    expected: "map".into(),
                    actual: format!("{node:?}").into(),
                }));
            };

            // Extract meta field which contains user data.
            node_map.get("meta").map(|meta| T::hydrate(meta))
        })
        .collect()
}
