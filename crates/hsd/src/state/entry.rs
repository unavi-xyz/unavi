/// Orders concurrent writes to one key exactly as `single_latest_per_key`
/// does at read time, so a live-applied entry and a re-read of the same
/// document agree on the winner.
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub key:       String,
    pub value:     Vec<u8>,
    pub timestamp: u64,
}

impl Entry {
    #[must_use]
    pub fn new(key: impl Into<String>, value: Vec<u8>, timestamp: u64) -> Self {
        Self {
            key: key.into(),
            value,
            timestamp,
        }
    }

    #[must_use]
    pub fn bytes(key: impl Into<String>, value: impl Into<Vec<u8>>, timestamp: u64) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            timestamp,
        }
    }
}
