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
    List(Vec<RecordValue>),
    Map(Vec<(String, RecordValue)>),
}

#[cfg(feature = "loro")]
impl From<loro::LoroValue> for RecordValue {
    fn from(v: loro::LoroValue) -> Self {
        match v {
            loro::LoroValue::Null => RecordValue::Null,
            loro::LoroValue::Bool(b) => RecordValue::Bool(b),
            loro::LoroValue::I64(n) => RecordValue::I64(n),
            loro::LoroValue::Double(n) => RecordValue::F64(n),
            loro::LoroValue::String(s) => RecordValue::String(s.to_string()),
            loro::LoroValue::Binary(b) => RecordValue::Binary(b.to_vec()),
            loro::LoroValue::List(list) => {
                RecordValue::List(list.iter().cloned().map(RecordValue::from).collect())
            }
            loro::LoroValue::Map(map) => RecordValue::Map(
                map.iter()
                    .map(|(k, v)| (k.clone(), RecordValue::from(v.clone())))
                    .collect(),
            ),
            loro::LoroValue::Container(_) => RecordValue::Null,
        }
    }
}
