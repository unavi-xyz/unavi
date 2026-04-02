use serde::{Deserialize, Serialize};

/// Portable representation of a Loro container value for WASM transport.
///
/// Serialized with postcard for cross-boundary delivery.
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

#[cfg(feature = "loro")]
impl From<loro::LoroValue> for RecordValue {
    fn from(v: loro::LoroValue) -> Self {
        match v {
            loro::LoroValue::Bool(b) => Self::Bool(b),
            loro::LoroValue::I64(n) => Self::I64(n),
            loro::LoroValue::Double(n) => Self::F64(n),
            loro::LoroValue::String(s) => Self::String(s.to_string()),
            loro::LoroValue::Binary(b) => Self::Binary(b.to_vec()),
            loro::LoroValue::List(list) => {
                Self::List(list.iter().cloned().map(Self::from).collect())
            }
            loro::LoroValue::Map(map) => Self::Map(
                map.iter()
                    .map(|(k, v)| (k.clone(), Self::from(v.clone())))
                    .collect(),
            ),
            loro::LoroValue::Null | loro::LoroValue::Container(_) => Self::Null,
        }
    }
}
