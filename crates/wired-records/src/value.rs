use loro::LoroValue;
use serde::{
    Deserialize,
    Serialize,
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
