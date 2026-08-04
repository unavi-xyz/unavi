use crate::id::BlobId;

/// A bulk entry as seen from state: the hash and size iroh-docs stores, never
/// the bytes. Those come from the blob store through the asset path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BulkRef {
    pub hash: BlobId,
    pub size: u64,
}

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

    #[must_use]
    pub const fn from_hash(timestamp: u64, hash: BlobId) -> Self {
        Self {
            timestamp,
            content: hash.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryValue {
    /// A `p/` or `meta/` value, fetched eagerly because it is small.
    Bytes(Vec<u8>),
    /// A `b/` value, left in the blob store.
    Blob(BulkRef),
}

impl EntryValue {
    /// An empty value is how a peer removes a key it does not author: `del`
    /// only sweeps the caller's own entries, so cross-author removal has to be
    /// expressed as data.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        match self {
            Self::Bytes(bytes) => bytes.is_empty(),
            Self::Blob(bulk) => bulk.size == 0,
        }
    }

    #[must_use]
    pub fn stamp(&self, timestamp: u64) -> Stamp {
        match self {
            Self::Bytes(bytes) => Stamp::new(timestamp, bytes),
            Self::Blob(bulk) => Stamp::from_hash(timestamp, bulk.hash),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub key:       String,
    pub value:     EntryValue,
    pub timestamp: u64,
}

impl Entry {
    #[must_use]
    pub fn bytes(key: impl Into<String>, value: impl Into<Vec<u8>>, timestamp: u64) -> Self {
        Self {
            key: key.into(),
            value: EntryValue::Bytes(value.into()),
            timestamp,
        }
    }

    #[must_use]
    pub fn blob(key: impl Into<String>, hash: BlobId, size: u64, timestamp: u64) -> Self {
        Self {
            key: key.into(),
            value: EntryValue::Blob(BulkRef { hash, size }),
            timestamp,
        }
    }
}
