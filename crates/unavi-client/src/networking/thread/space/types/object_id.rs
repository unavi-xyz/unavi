//! Unique identifier for dynamic objects within a space.

use blake3::Hash;
use postcard::experimental::max_size::MaxSize;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

/// Maximum byte length for a node name in an ObjectId.
const MAX_NODE_NAME_LEN: usize = 64;

/// Unique identifier for a dynamic object.
///
/// Composed of a WDS record ID (blake3 hash) and an index within that record.
/// This allows multiple objects to be defined within a single record.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId {
    /// WDS record ID (blake3 hash).
    pub record: Hash,
    /// Node within the record.
    pub node: SmolStr,
}

impl ObjectId {
    pub const fn new(record: Hash, node: SmolStr) -> Self {
        Self { record, node }
    }
}

impl MaxSize for ObjectId {
    // 32 bytes for blake3 hash + varint length + node name bytes.
    const POSTCARD_MAX_SIZE: usize = 32 + 1 + MAX_NODE_NAME_LEN;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let record = blake3::hash(b"test record");
        let id = ObjectId::new(record, "alice".into());

        assert_eq!(id.record, record);
        assert_eq!(id.node, "alice");
        assert_eq!(id.record, record);
    }

    #[test]
    fn test_equality() {
        let record = blake3::hash(b"test record");
        let id1 = ObjectId::new(record, "alice".into());
        let id2 = ObjectId::new(record, "alice".into());
        let id3 = ObjectId::new(record, "bob".into());

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_different_records() {
        let record1 = blake3::hash(b"record 1");
        let record2 = blake3::hash(b"record 2");

        let id1 = ObjectId::new(record1, "alice".into());
        let id2 = ObjectId::new(record2, "alice".into());

        assert_ne!(id1, id2);
    }
}
