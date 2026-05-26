use std::collections::{BTreeMap, HashMap};

use loro::{Container, LoroList, LoroMap, LoroMovableList, LoroValue, ValueOrContainer};

use crate::{error::HydrateError, hydrate::Hydrate};

impl Hydrate for bool {
    fn hydrate_bool(b: bool) -> Result<Self, HydrateError> {
        Ok(b)
    }
}

macro_rules! impl_hydrate_signed {
    ($($t:ty),*) => {
        $(
            impl Hydrate for $t {
                fn hydrate_i64(i: i64) -> Result<Self, HydrateError> {
                    <$t>::try_from(i).map_err(|_| HydrateError::Overflow {
                        value: i,
                        target_type: stringify!($t),
                    })
                }
            }
        )*
    };
}

impl_hydrate_signed!(i8, i16, i32);

impl Hydrate for i64 {
    fn hydrate_i64(i: i64) -> Result<Self, HydrateError> {
        Ok(i)
    }
}

macro_rules! impl_hydrate_unsigned {
    ($($t:ty),*) => {
        $(
            impl Hydrate for $t {
                fn hydrate_i64(i: i64) -> Result<Self, HydrateError> {
                    <$t>::try_from(i).map_err(|_| HydrateError::Overflow {
                        value: i,
                        target_type: stringify!($t),
                    })
                }
            }
        )*
    };
}

impl_hydrate_unsigned!(u8, u16, u32, u64);

impl Hydrate for usize {
    fn hydrate_i64(i: i64) -> Result<Self, HydrateError> {
        Self::try_from(i).map_err(|_| HydrateError::Overflow {
            value: i,
            target_type: "usize",
        })
    }
}

impl Hydrate for f64 {
    fn hydrate_f64(f: f64) -> Result<Self, HydrateError> {
        Ok(f)
    }

    fn hydrate_i64(i: i64) -> Result<Self, HydrateError> {
        Ok(i as Self)
    }
}

impl Hydrate for f32 {
    fn hydrate_f64(f: f64) -> Result<Self, HydrateError> {
        Ok(f as Self)
    }

    fn hydrate_i64(i: i64) -> Result<Self, HydrateError> {
        Ok(i as Self)
    }
}

impl Hydrate for String {
    fn hydrate_string(s: &str) -> Result<Self, HydrateError> {
        Ok(s.to_string())
    }
}

impl<T: Hydrate> Hydrate for Option<T> {
    fn hydrate(source: &ValueOrContainer) -> Result<Self, HydrateError> {
        match source {
            ValueOrContainer::Value(LoroValue::Null) => Ok(None),
            other => T::hydrate(other).map(Some),
        }
    }

    fn hydrate_null() -> Result<Self, HydrateError> {
        Ok(None)
    }

    fn hydrate_bool(b: bool) -> Result<Self, HydrateError> {
        T::hydrate_bool(b).map(Some)
    }

    fn hydrate_i64(i: i64) -> Result<Self, HydrateError> {
        T::hydrate_i64(i).map(Some)
    }

    fn hydrate_f64(f: f64) -> Result<Self, HydrateError> {
        T::hydrate_f64(f).map(Some)
    }

    fn hydrate_string(s: &str) -> Result<Self, HydrateError> {
        T::hydrate_string(s).map(Some)
    }

    fn hydrate_binary(b: &[u8]) -> Result<Self, HydrateError> {
        T::hydrate_binary(b).map(Some)
    }

    fn hydrate_inline_list(items: &[LoroValue]) -> Result<Self, HydrateError> {
        T::hydrate_inline_list(items).map(Some)
    }

    fn hydrate_map(map: &LoroMap) -> Result<Self, HydrateError> {
        T::hydrate_map(map).map(Some)
    }

    fn hydrate_list(list: &LoroList) -> Result<Self, HydrateError> {
        T::hydrate_list(list).map(Some)
    }

    fn hydrate_movable_list(list: &LoroMovableList) -> Result<Self, HydrateError> {
        T::hydrate_movable_list(list).map(Some)
    }
}

impl<T: Hydrate> Hydrate for Vec<T> {
    fn hydrate_list(list: &LoroList) -> Result<Self, HydrateError> {
        let mut out = Self::with_capacity(list.len());
        for i in 0..list.len() {
            match list.get(i) {
                Some(voc) => out.push(T::hydrate(&voc)?),
                None => return Err(HydrateError::missing(format!("[{i}]"))),
            }
        }
        Ok(out)
    }

    fn hydrate_movable_list(list: &LoroMovableList) -> Result<Self, HydrateError> {
        let mut out = Self::with_capacity(list.len());
        for i in 0..list.len() {
            match list.get(i) {
                Some(voc) => out.push(T::hydrate(&voc)?),
                None => return Err(HydrateError::missing(format!("[{i}]"))),
            }
        }
        Ok(out)
    }

    fn hydrate_inline_list(items: &[LoroValue]) -> Result<Self, HydrateError> {
        items.iter().map(|v| T::hydrate_value(v)).collect()
    }
}

impl<T: Hydrate, const N: usize> Hydrate for [T; N] {
    fn hydrate_inline_list(items: &[LoroValue]) -> Result<Self, HydrateError> {
        if items.len() != N {
            return Err(HydrateError::unexpected(
                "inline list of correct length",
                "inline list of wrong length",
            ));
        }
        let mut vec: Vec<T> = Vec::with_capacity(N);
        for item in items {
            vec.push(T::hydrate_value(item)?);
        }
        vec.try_into()
            .map_err(|_: Vec<T>| HydrateError::unexpected("array of size N", "wrong size"))
    }

    fn hydrate_list(list: &LoroList) -> Result<Self, HydrateError> {
        if list.len() != N {
            return Err(HydrateError::unexpected(
                "list of correct length",
                "list of wrong length",
            ));
        }
        let mut vec: Vec<T> = Vec::with_capacity(N);
        for i in 0..N {
            match list.get(i) {
                Some(voc) => vec.push(T::hydrate(&voc)?),
                None => return Err(HydrateError::missing(format!("[{i}]"))),
            }
        }
        vec.try_into()
            .map_err(|_: Vec<T>| HydrateError::unexpected("array of size N", "wrong size"))
    }
}

impl<V: Hydrate, S: std::hash::BuildHasher + Default> Hydrate for HashMap<String, V, S> {
    fn hydrate_map(map: &LoroMap) -> Result<Self, HydrateError> {
        hydrate_string_map(map)
    }
}

impl<V: Hydrate> Hydrate for BTreeMap<String, V> {
    fn hydrate_map(map: &LoroMap) -> Result<Self, HydrateError> {
        let hm: HashMap<String, V> = hydrate_string_map(map)?;
        Ok(hm.into_iter().collect())
    }
}

fn hydrate_string_map<V: Hydrate, M: FromIterator<(String, V)>>(
    map: &LoroMap,
) -> Result<M, HydrateError> {
    let mut pairs = Vec::new();
    map.for_each(|key, voc| {
        pairs.push((key.to_string(), voc));
    });
    pairs
        .into_iter()
        .map(|(k, voc)| V::hydrate(&voc).map(|v| (k, v)))
        .collect()
}

pub fn hydrate_keyed_map<K, V>(map: &LoroMap) -> Result<HashMap<K, V>, HydrateError>
where
    K: From<String> + Eq + std::hash::Hash,
    V: Hydrate,
{
    let mut pairs = Vec::new();
    map.for_each(|key, voc| {
        pairs.push((key.to_string(), voc));
    });
    pairs
        .into_iter()
        .map(|(k, voc)| V::hydrate(&voc).map(|v| (K::from(k), v)))
        .collect()
}

impl<T: Hydrate> Hydrate for Box<T> {
    fn hydrate(source: &ValueOrContainer) -> Result<Self, HydrateError> {
        T::hydrate(source).map(Self::new)
    }
    fn hydrate_value(value: &LoroValue) -> Result<Self, HydrateError> {
        T::hydrate_value(value).map(Self::new)
    }
    fn hydrate_map(map: &LoroMap) -> Result<Self, HydrateError> {
        T::hydrate_map(map).map(Self::new)
    }
    fn hydrate_list(list: &LoroList) -> Result<Self, HydrateError> {
        T::hydrate_list(list).map(Self::new)
    }
    fn hydrate_movable_list(list: &LoroMovableList) -> Result<Self, HydrateError> {
        T::hydrate_movable_list(list).map(Self::new)
    }
    fn hydrate_null() -> Result<Self, HydrateError> {
        T::hydrate_null().map(Self::new)
    }
    fn hydrate_bool(b: bool) -> Result<Self, HydrateError> {
        T::hydrate_bool(b).map(Self::new)
    }
    fn hydrate_i64(i: i64) -> Result<Self, HydrateError> {
        T::hydrate_i64(i).map(Self::new)
    }
    fn hydrate_f64(f: f64) -> Result<Self, HydrateError> {
        T::hydrate_f64(f).map(Self::new)
    }
    fn hydrate_string(s: &str) -> Result<Self, HydrateError> {
        T::hydrate_string(s).map(Self::new)
    }
    fn hydrate_binary(b: &[u8]) -> Result<Self, HydrateError> {
        T::hydrate_binary(b).map(Self::new)
    }
    fn hydrate_inline_list(items: &[LoroValue]) -> Result<Self, HydrateError> {
        T::hydrate_inline_list(items).map(Self::new)
    }
}

impl Hydrate for LoroValue {
    fn hydrate(source: &ValueOrContainer) -> Result<Self, HydrateError> {
        match source {
            ValueOrContainer::Value(v) => Ok(v.clone()),
            ValueOrContainer::Container(c) => Ok(match c {
                Container::Map(m) => m.get_deep_value(),
                Container::List(l) => l.get_deep_value(),
                Container::MovableList(l) => l.get_deep_value(),
                _ => return Err(HydrateError::unexpected("known container", "unknown")),
            }),
        }
    }

    fn hydrate_value(value: &LoroValue) -> Result<Self, HydrateError> {
        Ok(value.clone())
    }

    fn hydrate_map(map: &LoroMap) -> Result<Self, HydrateError> {
        Ok(map.get_deep_value())
    }

    fn hydrate_list(list: &LoroList) -> Result<Self, HydrateError> {
        Ok(list.get_deep_value())
    }

    fn hydrate_movable_list(list: &LoroMovableList) -> Result<Self, HydrateError> {
        Ok(list.get_deep_value())
    }

    fn hydrate_null() -> Result<Self, HydrateError> {
        Ok(Self::Null)
    }

    fn hydrate_bool(b: bool) -> Result<Self, HydrateError> {
        Ok(Self::Bool(b))
    }

    fn hydrate_i64(i: i64) -> Result<Self, HydrateError> {
        Ok(Self::I64(i))
    }

    fn hydrate_f64(f: f64) -> Result<Self, HydrateError> {
        Ok(Self::Double(f))
    }

    fn hydrate_string(s: &str) -> Result<Self, HydrateError> {
        Ok(Self::String(s.to_string().into()))
    }

    fn hydrate_binary(b: &[u8]) -> Result<Self, HydrateError> {
        Ok(Self::Binary(b.to_vec().into()))
    }
}

impl Hydrate for serde_json::Value {
    fn hydrate_null() -> Result<Self, HydrateError> {
        Ok(Self::Null)
    }

    fn hydrate_bool(b: bool) -> Result<Self, HydrateError> {
        Ok(Self::Bool(b))
    }

    fn hydrate_i64(i: i64) -> Result<Self, HydrateError> {
        Ok(Self::Number(i.into()))
    }

    fn hydrate_f64(f: f64) -> Result<Self, HydrateError> {
        Ok(serde_json::Number::from_f64(f).map_or(Self::Null, serde_json::Value::Number))
    }

    fn hydrate_string(s: &str) -> Result<Self, HydrateError> {
        Ok(Self::String(s.to_string()))
    }

    fn hydrate_map(map: &LoroMap) -> Result<Self, HydrateError> {
        let deep = map.get_deep_value();
        Ok(loro_value_to_json(&deep))
    }

    fn hydrate_list(list: &LoroList) -> Result<Self, HydrateError> {
        let deep = list.get_deep_value();
        Ok(loro_value_to_json(&deep))
    }
}

fn loro_value_to_json(v: &LoroValue) -> serde_json::Value {
    match v {
        LoroValue::Null | LoroValue::Container(_) => serde_json::Value::Null,
        LoroValue::Bool(b) => serde_json::Value::Bool(*b),
        LoroValue::I64(i) => serde_json::Value::Number((*i).into()),
        LoroValue::Double(f) => serde_json::Number::from_f64(*f)
            .map_or(serde_json::Value::Null, serde_json::Value::Number),
        LoroValue::String(s) => serde_json::Value::String(s.to_string()),
        LoroValue::Binary(b) => serde_json::Value::Array(
            b.iter()
                .map(|byte| serde_json::Value::Number((i64::from(*byte)).into()))
                .collect(),
        ),
        LoroValue::List(list) => {
            serde_json::Value::Array(list.iter().map(loro_value_to_json).collect())
        }
        LoroValue::Map(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), loro_value_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
    }
}
