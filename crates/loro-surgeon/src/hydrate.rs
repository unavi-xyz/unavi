use std::collections::BTreeMap;

use loro::{LoroMapValue, LoroValue};
use smol_str::SmolStr;

use crate::HydrateError;

/// Read a Rust value from a Loro value.
pub trait Hydrate: Sized {
    /// Hydrate a value from a [`LoroValue`].
    ///
    /// # Errors
    ///
    /// Returns [`HydrateError`] if the value type doesn't match or required
    /// fields are missing.
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError>;
}

impl Hydrate for LoroValue {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        Ok(value.clone())
    }
}

impl Hydrate for LoroMapValue {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        match value {
            LoroValue::Map(map) => Ok(map.clone()),
            _ => Err(HydrateError::TypeMismatch {
                path: SmolStr::default(),
                expected: "Map".into(),
                actual: format!("{value:?}").into(),
            }),
        }
    }
}

impl Hydrate for bool {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        match value {
            LoroValue::Bool(b) => Ok(*b),
            _ => Err(HydrateError::TypeMismatch {
                path: SmolStr::default(),
                expected: "Bool".into(),
                actual: format!("{value:?}").into(),
            }),
        }
    }
}

impl Hydrate for i64 {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        match value {
            LoroValue::I64(n) => Ok(*n),
            _ => Err(HydrateError::TypeMismatch {
                path: SmolStr::default(),
                expected: "I64".into(),
                actual: format!("{value:?}").into(),
            }),
        }
    }
}

impl Hydrate for f64 {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        match value {
            LoroValue::Double(n) => Ok(*n),
            _ => Err(HydrateError::TypeMismatch {
                path: SmolStr::default(),
                expected: "Double".into(),
                actual: format!("{value:?}").into(),
            }),
        }
    }
}

impl Hydrate for String {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        match value {
            LoroValue::String(s) => Ok(s.to_string()),
            _ => Err(HydrateError::TypeMismatch {
                path: SmolStr::default(),
                expected: "String".into(),
                actual: format!("{value:?}").into(),
            }),
        }
    }
}

impl Hydrate for SmolStr {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        match value {
            LoroValue::String(s) => Ok(s.as_str().into()),
            _ => Err(HydrateError::TypeMismatch {
                path: Self::default(),
                expected: "String".into(),
                actual: format!("{value:?}").into(),
            }),
        }
    }
}

impl Hydrate for Vec<u8> {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        match value {
            LoroValue::Binary(b) => Ok(b.to_vec()),
            _ => Err(HydrateError::TypeMismatch {
                path: SmolStr::default(),
                expected: "Binary".into(),
                actual: format!("{value:?}").into(),
            }),
        }
    }
}

impl<T: Hydrate> Hydrate for Option<T> {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        match value {
            LoroValue::Null => Ok(None),
            _ => Ok(Some(T::hydrate(value)?)),
        }
    }
}

impl<T: Hydrate> Hydrate for Vec<T> {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        match value {
            LoroValue::List(list) => list.iter().map(T::hydrate).collect(),
            _ => Err(HydrateError::TypeMismatch {
                path: SmolStr::default(),
                expected: "List".into(),
                actual: format!("{value:?}").into(),
            }),
        }
    }
}

impl<V: Hydrate> Hydrate for BTreeMap<String, V> {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        match value {
            LoroValue::Map(map) => map
                .iter()
                .map(|(k, v)| Ok((k.clone(), V::hydrate(v)?)))
                .collect(),
            _ => Err(HydrateError::TypeMismatch {
                path: SmolStr::default(),
                expected: "Map".into(),
                actual: format!("{value:?}").into(),
            }),
        }
    }
}

impl<V: Hydrate> Hydrate for BTreeMap<SmolStr, V> {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        match value {
            LoroValue::Map(map) => map
                .iter()
                .map(|(k, v)| Ok((SmolStr::from(k.as_str()), V::hydrate(v)?)))
                .collect(),
            _ => Err(HydrateError::TypeMismatch {
                path: SmolStr::default(),
                expected: "Map".into(),
                actual: format!("{value:?}").into(),
            }),
        }
    }
}

impl Hydrate for f32 {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        match value {
            LoroValue::Double(n) => Ok(*n as f32),
            _ => Err(HydrateError::TypeMismatch {
                path: SmolStr::default(),
                expected: "Double".into(),
                actual: format!("{value:?}").into(),
            }),
        }
    }
}

impl<const N: usize> Hydrate for [u8; N] {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        match value {
            LoroValue::Binary(b) => {
                let slice: &[u8] = b.as_ref();
                slice.try_into().map_err(|_| {
                    HydrateError::Custom(format!("expected {} bytes, got {}", N, b.len()).into())
                })
            }
            _ => Err(HydrateError::TypeMismatch {
                path: SmolStr::default(),
                expected: "Binary".into(),
                actual: format!("{value:?}").into(),
            }),
        }
    }
}

impl<const N: usize> Hydrate for [f64; N] {
    fn hydrate(value: &LoroValue) -> Result<Self, HydrateError> {
        match value {
            LoroValue::List(list) => {
                let vec: Vec<f64> = list
                    .iter()
                    .map(|v| {
                        v.as_double()
                            .copied()
                            .ok_or_else(|| HydrateError::TypeMismatch {
                                path: "[list item]".into(),
                                expected: "f64".into(),
                                actual: format!("{v:?}").into(),
                            })
                    })
                    .collect::<Result<_, _>>()?;
                vec.try_into().map_err(|_| {
                    HydrateError::Custom(
                        format!("expected {} elements, got {}", N, list.len()).into(),
                    )
                })
            }
            _ => Err(HydrateError::TypeMismatch {
                path: SmolStr::default(),
                expected: "List".into(),
                actual: format!("{value:?}").into(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use loro::LoroDoc;
    use rstest::rstest;

    use super::*;

    fn unwrap_value(v: loro::ValueOrContainer) -> LoroValue {
        match v {
            loro::ValueOrContainer::Value(v) => v,
            loro::ValueOrContainer::Container(_) => panic!("expected value, got container"),
        }
    }

    #[rstest]
    fn hydrate_bool() {
        let doc = LoroDoc::new();
        let map = doc.get_map("test");
        map.insert("value", true).expect("insert failed");

        let value = unwrap_value(map.get("value").expect("missing value"));
        let result = bool::hydrate(&value).expect("hydrate failed");
        assert!(result);
    }

    #[rstest]
    fn hydrate_i64() {
        let doc = LoroDoc::new();
        let map = doc.get_map("test");
        map.insert("value", 42i64).expect("insert failed");

        let value = unwrap_value(map.get("value").expect("missing value"));
        let result = i64::hydrate(&value).expect("hydrate failed");
        assert_eq!(result, 42);
    }

    #[rstest]
    fn hydrate_string() {
        let doc = LoroDoc::new();
        let map = doc.get_map("test");
        map.insert("value", "hello").expect("insert failed");

        let value = unwrap_value(map.get("value").expect("missing value"));
        let result = String::hydrate(&value).expect("hydrate failed");
        assert_eq!(result, "hello");
    }

    #[rstest]
    fn hydrate_option_some() {
        let doc = LoroDoc::new();
        let map = doc.get_map("test");
        map.insert("value", "hello").expect("insert failed");

        let value = unwrap_value(map.get("value").expect("missing value"));
        let result = Option::<String>::hydrate(&value).expect("hydrate failed");
        assert_eq!(result, Some("hello".to_string()));
    }

    #[rstest]
    fn hydrate_option_none() {
        let value = LoroValue::Null;
        let result = Option::<String>::hydrate(&value).expect("hydrate failed");
        assert_eq!(result, None);
    }
}
