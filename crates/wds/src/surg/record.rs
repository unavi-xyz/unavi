use std::collections::BTreeMap;

use blake3::Hash;
use loro::LoroDoc;
use lorosurgeon::{Hydrate, HydrateError, Reconcile, ReconcileError, reconcile::RootReconciler};
use rand::Rng;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use wired_records::{byte_array::ByteArray, did::HydratedDid};
use wired_schemas::{SCHEMA_ACL, SCHEMA_RECORD};
use xdid::core::did::Did;

/// Fixed-size nonce for record identification.
pub type RecordNonce = ByteArray<16>;

/// A WDS record containing metadata about the document.
#[derive(Debug, Clone, Serialize, Deserialize, Hydrate, Reconcile)]
pub struct Record {
    pub creator: HydratedDid,
    pub nonce: RecordNonce,
    pub schemas: BTreeMap<String, ByteArray<32>>,
    pub timestamp: i64,
}

impl Record {
    /// Create a new record with default schemas.
    #[must_use]
    pub fn new(creator: Did) -> Self {
        let mut nonce = RecordNonce::default();
        rand::rng().fill(&mut nonce.0.0);

        let mut schemas = BTreeMap::new();
        schemas.insert("acl".to_string(), ByteArray::from(SCHEMA_ACL.hash));
        schemas.insert("record".to_string(), ByteArray::from(SCHEMA_RECORD.hash));

        Self {
            creator: HydratedDid(creator),
            nonce,
            schemas,
            timestamp: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }

    pub fn add_schema(&mut self, container: String, schema: blake3::Hash) {
        self.schemas.insert(container, schema.into());
    }

    pub fn id(&self) -> postcard::Result<Hash> {
        let bytes = postcard::to_stdvec(self)?;
        Ok(blake3::hash(&bytes))
    }

    pub fn save(&self, doc: &LoroDoc) -> Result<(), ReconcileError> {
        let map = doc.get_map("record");
        let rec = RootReconciler::new(map);
        self.reconcile(rec)
    }

    pub fn load(doc: &LoroDoc) -> Result<Self, HydrateError> {
        let map = doc.get_map("record");
        Self::hydrate_map(&map)
    }
}

#[cfg(test)]
mod tests {
    use loro::LoroDoc;
    use rstest::rstest;

    use super::*;

    fn test_did() -> Did {
        "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"
            .parse()
            .expect("valid did")
    }

    #[rstest]
    fn roundtrip_record() {
        let doc = LoroDoc::new();
        let record = Record::new(test_did());

        record.save(&doc).expect("save failed");
        let loaded = Record::load(&doc).expect("load failed");

        assert_eq!(record.creator.0, loaded.creator.0);
        assert_eq!(record.nonce.as_bytes(), loaded.nonce.as_bytes());
        assert_eq!(record.timestamp, loaded.timestamp);
        assert_eq!(record.schemas.len(), loaded.schemas.len());
    }
}
