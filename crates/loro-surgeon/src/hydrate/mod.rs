//! Read Rust types from Loro containers.

pub mod impls;

use loro::{Container, LoroDoc, LoroList, LoroMap, LoroMovableList, LoroValue, ValueOrContainer};

use crate::error::HydrateError;

pub trait Hydrate: Sized {
    fn hydrate(source: &ValueOrContainer) -> Result<Self, HydrateError> {
        match source {
            ValueOrContainer::Value(v) => Self::hydrate_value(v),
            ValueOrContainer::Container(c) => match c {
                Container::Map(m) => Self::hydrate_map(m),
                Container::List(l) => Self::hydrate_list(l),
                Container::MovableList(l) => Self::hydrate_movable_list(l),
                _ => Err(HydrateError::unexpected("known container", "unknown")),
            },
        }
    }

    fn hydrate_value(value: &LoroValue) -> Result<Self, HydrateError> {
        match value {
            LoroValue::Null => Self::hydrate_null(),
            LoroValue::Bool(b) => Self::hydrate_bool(*b),
            LoroValue::I64(i) => Self::hydrate_i64(*i),
            LoroValue::Double(f) => Self::hydrate_f64(*f),
            LoroValue::String(s) => Self::hydrate_string(s),
            LoroValue::Binary(b) => Self::hydrate_binary(b),
            LoroValue::List(items) => Self::hydrate_inline_list(items),
            LoroValue::Map(_) => Err(HydrateError::unexpected("scalar", "inline map")),
            LoroValue::Container(_) => Err(HydrateError::unexpected("scalar", "container ref")),
        }
    }

    fn hydrate_map(map: &LoroMap) -> Result<Self, HydrateError> {
        let _ = map;
        Err(HydrateError::unexpected("other", "map"))
    }

    fn hydrate_list(list: &LoroList) -> Result<Self, HydrateError> {
        let _ = list;
        Err(HydrateError::unexpected("other", "list"))
    }

    fn hydrate_movable_list(list: &LoroMovableList) -> Result<Self, HydrateError> {
        let _ = list;
        Err(HydrateError::unexpected("other", "movable_list"))
    }

    fn hydrate_null() -> Result<Self, HydrateError> {
        Err(HydrateError::unexpected("other", "null"))
    }

    fn hydrate_bool(_b: bool) -> Result<Self, HydrateError> {
        Err(HydrateError::unexpected("other", "bool"))
    }

    fn hydrate_i64(_i: i64) -> Result<Self, HydrateError> {
        Err(HydrateError::unexpected("other", "i64"))
    }

    fn hydrate_f64(_f: f64) -> Result<Self, HydrateError> {
        Err(HydrateError::unexpected("other", "f64"))
    }

    fn hydrate_string(_s: &str) -> Result<Self, HydrateError> {
        Err(HydrateError::unexpected("other", "string"))
    }

    fn hydrate_binary(_b: &[u8]) -> Result<Self, HydrateError> {
        Err(HydrateError::unexpected("other", "binary"))
    }

    /// Read from an atomic [`LoroValue::List`] (inline values, not a container).
    /// Used by `[T; N]` fixed-size arrays.
    fn hydrate_inline_list(_items: &[LoroValue]) -> Result<Self, HydrateError> {
        Err(HydrateError::unexpected("other", "inline list"))
    }
}

pub fn hydrate<T: Hydrate>(doc: &LoroDoc, root_key: &str) -> Result<T, HydrateError> {
    let map = doc.get_map(root_key);
    T::hydrate_map(&map)
}

pub fn hydrate_map<T: Hydrate>(map: &LoroMap) -> Result<T, HydrateError> {
    T::hydrate_map(map)
}

pub fn hydrate_prop<T: Hydrate>(map: &LoroMap, key: &str) -> Result<T, HydrateError> {
    map.get(key).map_or_else(
        || T::hydrate_null().map_err(|_| HydrateError::missing(key)),
        |voc| T::hydrate(&voc),
    )
}

pub fn hydrate_prop_or_default<T: Hydrate + Default>(
    map: &LoroMap,
    key: &str,
) -> Result<T, HydrateError> {
    map.get(key)
        .map_or_else(|| Ok(T::default()), |voc| T::hydrate(&voc))
}

pub fn hydrate_prop_or<T: Hydrate>(
    map: &LoroMap,
    key: &str,
    default: T,
) -> Result<T, HydrateError> {
    map.get(key).map_or(Ok(default), |voc| T::hydrate(&voc))
}

pub fn hydrate_prop_or_else<T: Hydrate>(
    map: &LoroMap,
    key: &str,
    default: impl FnOnce() -> T,
) -> Result<T, HydrateError> {
    map.get(key)
        .map_or_else(|| Ok(default()), |voc| T::hydrate(&voc))
}

pub fn hydrate_prop_json<T: serde::de::DeserializeOwned>(
    map: &LoroMap,
    key: &str,
) -> Result<T, HydrateError> {
    match map.get(key) {
        Some(ValueOrContainer::Value(LoroValue::String(s))) => {
            serde_json::from_str(&s).map_err(|e| HydrateError::Json {
                key: key.to_string(),
                source: e,
            })
        }
        Some(_) => Err(HydrateError::unexpected("string (json)", "other")),
        None => Err(HydrateError::missing(key)),
    }
}

pub fn hydrate_prop_json_or_default<T: serde::de::DeserializeOwned + Default>(
    map: &LoroMap,
    key: &str,
) -> Result<T, HydrateError> {
    match map.get(key) {
        Some(ValueOrContainer::Value(LoroValue::String(s))) => {
            serde_json::from_str(&s).map_err(|e| HydrateError::Json {
                key: key.to_string(),
                source: e,
            })
        }
        Some(_) => Err(HydrateError::unexpected("string (json)", "other")),
        None => Ok(T::default()),
    }
}

pub fn hydrate_list_item<T: Hydrate>(list: &LoroList, index: usize) -> Result<T, HydrateError> {
    list.get(index).map_or_else(
        || Err(HydrateError::missing(format!("[{index}]"))),
        |voc| T::hydrate(&voc),
    )
}

