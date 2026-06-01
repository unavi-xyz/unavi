use std::fmt;

use serde::{
    Deserialize,
    Serialize,
    de::{
        self,
        DeserializeOwned,
        Deserializer,
        IntoDeserializer,
        Visitor,
    },
};

/// Portable representation of a Loro container value for WASM transport.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecordValue {
    Null,
    Bool(bool),
    I64(i64),
    F64(f64),
    String(String),
    Binary(Vec<u8>),
    List(Vec<Self>),
    Map(Vec<(String, Self)>),
}

impl RecordValue {
    /// Deserialize this value into any serde-compatible type.
    pub fn into_typed<T: DeserializeOwned>(self) -> Result<T, RecordValueError> {
        T::deserialize(self)
    }

    /// Look up a key in a [`RecordValue::Map`], returning `None` for other
    /// variants.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Self> {
        if let Self::Map(fields) = self {
            fields.iter().find_map(|(k, v)| (k == key).then_some(v))
        } else {
            None
        }
    }
}

#[cfg(feature = "loro")]
mod loro_impls {
    use loro::LoroValue;

    use super::RecordValue;

    impl From<LoroValue> for RecordValue {
        fn from(v: LoroValue) -> Self {
            match v {
                LoroValue::Bool(b) => Self::Bool(b),
                LoroValue::I64(n) => Self::I64(n),
                LoroValue::Double(n) => Self::F64(n),
                LoroValue::String(s) => Self::String(s.to_string()),
                LoroValue::Binary(b) => Self::Binary(b.to_vec()),
                LoroValue::List(list) => Self::List(list.iter().cloned().map(Self::from).collect()),
                LoroValue::Map(map) => Self::Map(
                    map.iter()
                        .map(|(k, v)| (k.clone(), Self::from(v.clone())))
                        .collect(),
                ),
                LoroValue::Null | LoroValue::Container(_) => Self::Null,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecordValueError(String);

impl fmt::Display for RecordValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RecordValueError {}

impl de::Error for RecordValueError {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self(msg.to_string())
    }
}

impl<'de> Deserializer<'de> for RecordValue {
    type Error = RecordValueError;

    fn deserialize_any<V: Visitor<'de>>(self, v: V) -> Result<V::Value, Self::Error> {
        match self {
            Self::Null => v.visit_unit(),
            Self::Bool(b) => v.visit_bool(b),
            Self::I64(n) => v.visit_i64(n),
            Self::F64(n) => v.visit_f64(n),
            Self::String(s) => v.visit_string(s),
            Self::Binary(b) => v.visit_seq(ByteSeqAccess {
                iter: b.into_iter(),
            }),
            Self::List(items) => v.visit_seq(SeqAccess {
                iter: items.into_iter(),
            }),
            Self::Map(fields) => v.visit_map(MapAccess {
                iter:  fields.into_iter(),
                value: None,
            }),
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, v: V) -> Result<V::Value, Self::Error> {
        match self {
            Self::Null => v.visit_none(),
            other => v.visit_some(other),
        }
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, v: V) -> Result<V::Value, Self::Error> {
        match self {
            Self::Binary(b) => v.visit_byte_buf(b),
            other => other.deserialize_any(v),
        }
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, v: V) -> Result<V::Value, Self::Error> {
        self.deserialize_bytes(v)
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        _len: usize,
        v: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_seq(v)
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _len: usize,
        v: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_seq(v)
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        v: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_map(v)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        v: V,
    ) -> Result<V::Value, Self::Error> {
        v.visit_newtype_struct(self)
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        v: V,
    ) -> Result<V::Value, Self::Error> {
        self.deserialize_unit(v)
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, v: V) -> Result<V::Value, Self::Error> {
        self.deserialize_any(v)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        unit seq map identifier enum
    }
}

struct ByteSeqAccess {
    iter: std::vec::IntoIter<u8>,
}

impl<'de> de::SeqAccess<'de> for ByteSeqAccess {
    type Error = RecordValueError;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        self.iter.next().map_or(Ok(None), |byte| {
            seed.deserialize(byte.into_deserializer()).map(Some)
        })
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

struct SeqAccess {
    iter: std::vec::IntoIter<RecordValue>,
}

impl<'de> de::SeqAccess<'de> for SeqAccess {
    type Error = RecordValueError;

    fn next_element_seed<T: de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        self.iter
            .next()
            .map_or(Ok(None), |value| seed.deserialize(value).map(Some))
    }
}

struct MapAccess {
    iter:  std::vec::IntoIter<(String, RecordValue)>,
    value: Option<RecordValue>,
}

impl<'de> de::MapAccess<'de> for MapAccess {
    type Error = RecordValueError;

    fn next_key_seed<K: de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(key.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V: de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        let value = self
            .value
            .take()
            .ok_or_else(|| <RecordValueError as de::Error>::custom("value missing"))?;
        seed.deserialize(value)
    }
}
