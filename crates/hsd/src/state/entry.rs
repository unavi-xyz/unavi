use web_time::{
    SystemTime,
    UNIX_EPOCH,
};

use crate::property::Property;

/// Orders concurrent writes to one key exactly as iroh-docs orders entries at
/// read time, so a live-applied entry and a re-read of the same document
/// agree on the winner.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stamp {
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

    /// The current wall-clock timestamp, for a local write's stamp.
    #[must_use]
    pub fn now(value: &[u8]) -> Self {
        Self::new(now_micros(), value)
    }

    /// The content hash of `property`'s wire form, without encoding it to a
    /// scratch `Vec` first.
    #[must_use]
    pub fn for_property(property: &Property) -> Self {
        let mut hasher = blake3::Hasher::new();
        property.hash_into(&mut hasher);
        Self {
            timestamp: now_micros(),
            content:   *hasher.finalize().as_bytes(),
        }
    }

    /// The content hash of an attribute payload's wire form, without encoding
    /// it to a scratch `Vec` first.
    #[must_use]
    pub fn for_attribute(payload: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        Property::hash_attribute_into(&mut hasher, payload);
        Self {
            timestamp: now_micros(),
            content:   *hasher.finalize().as_bytes(),
        }
    }
}

fn now_micros() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_micros() as u64)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub key:       String,
    pub value:     Vec<u8>,
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
