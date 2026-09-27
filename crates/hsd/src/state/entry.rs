use web_time::{
    SystemTime,
    UNIX_EPOCH,
};

use crate::property::value::Value;

/// Orders the opinions a live layer holds on one key. The later timestamp
/// wins, then the greater content hash.
///
/// A projected opinion carries the stamp of the entry its store chose. Every
/// peer holding that entry derives the same stamp, which is what a parent
/// cycle breaks on.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stamp {
    /// Milliseconds since the Unix epoch.
    pub timestamp: u64,
    pub content:   [u8; 32],
}

impl Stamp {
    #[must_use]
    pub fn new(timestamp: u64, value: &[u8]) -> Self {
        Self {
            timestamp,
            content: *blake3::hash(value).as_bytes(),
        }
    }

    /// Equal to [`Self::new`] over `property`'s encoding, without building it.
    #[must_use]
    pub fn of_property(timestamp: u64, property: &Value) -> Self {
        let mut hasher = blake3::Hasher::new();
        property.hash_into(&mut hasher);
        Self {
            timestamp,
            content: *hasher.finalize().as_bytes(),
        }
    }

    /// Equal to [`Self::of_property`] over an attribute holding `payload`.
    #[must_use]
    pub fn of_attribute(timestamp: u64, payload: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        Value::hash_attribute_into(&mut hasher, payload);
        Self {
            timestamp,
            content: *hasher.finalize().as_bytes(),
        }
    }
}

pub(super) fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// One key and its value. An empty value is a key holding no value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub key:       String,
    pub value:     Vec<u8>,
    /// Milliseconds since the Unix epoch.
    pub timestamp: u64,
}

impl Entry {
    #[must_use]
    pub fn new(key: impl Into<String>, value: impl Into<Vec<u8>>, timestamp: u64) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            timestamp,
        }
    }
}
