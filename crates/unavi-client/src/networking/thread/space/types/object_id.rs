//! Unique identifier for dynamic objects within a space.

use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};

/// Unique identifier for a dynamic object.
///
/// Composed of a WDS record ID (blake3 hash) and an index within that record.
/// This allows multiple objects to be defined within a single record.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId {
    /// WDS record ID (blake3 hash).
    pub record: [u8; 32],
    /// Index within the record.
    pub index: i64,
}

impl MaxSize for ObjectId {
    const POSTCARD_MAX_SIZE: usize = 32 + 8; // record + i64
}

impl ObjectId {
    pub const fn new(record: blake3::Hash, index: i64) -> Self {
        Self {
            record: *record.as_bytes(),
            index,
        }
    }

    /// Get the record hash.
    pub const fn record_hash(&self) -> blake3::Hash {
        blake3::Hash::from_bytes(self.record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let record = blake3::hash(b"test record");
        let id = ObjectId::new(record, 42);

        assert_eq!(id.record, *record.as_bytes());
        assert_eq!(id.index, 42);
        assert_eq!(id.record_hash(), record);
    }

    #[test]
    fn test_equality() {
        let record = blake3::hash(b"test record");
        let id1 = ObjectId::new(record, 0);
        let id2 = ObjectId::new(record, 0);
        let id3 = ObjectId::new(record, 1);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_different_records() {
        let record1 = blake3::hash(b"record 1");
        let record2 = blake3::hash(b"record 2");

        let id1 = ObjectId::new(record1, 0);
        let id2 = ObjectId::new(record2, 0);

        assert_ne!(id1, id2);
    }
}
