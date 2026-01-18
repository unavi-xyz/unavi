use std::collections::BTreeMap;

use blake3::Hash;
use loro::LoroDoc;
use loro_surgeon::{Hydrate, Reconcile};
use rand::Rng;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use time::OffsetDateTime;
use xdid::core::did::Did;

use crate::{conv, schemas};

/// Fixed-size nonce for record identification.
pub type RecordNonce = [u8; 16];

/// A WDS record containing metadata about the document.
#[derive(Debug, Clone, Serialize, Deserialize, Hydrate, Reconcile)]
pub struct Record {
    #[loro(with = "conv::did")]
    pub creator: Did,
    #[loro(with = "conv::byte_slice")]
    pub nonce: RecordNonce,
    #[loro(with = "conv::hash::map")]
    pub schemas: BTreeMap<SmolStr, Hash>,
    pub timestamp: i64,
}

impl Record {
    /// Create a new record with default schemas.
    #[must_use]
    pub fn new(creator: Did) -> Self {
        let mut nonce = RecordNonce::default();
        rand::rng().fill(&mut nonce);

        let mut schemas = BTreeMap::new();
        schemas.insert("acl".into(), schemas::SCHEMA_ACL.hash);
        schemas.insert("record".into(), schemas::SCHEMA_RECORD.hash);

        Self {
            creator,
            nonce,
            schemas,
            timestamp: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }

    pub fn add_schema(&mut self, container: SmolStr, schema: Hash) {
        self.schemas.insert(container, schema);
    }

    /// # Errors
    ///
    /// Returns an error if the record could not be serialized.
    pub fn id(&self) -> postcard::Result<Hash> {
        let bytes = postcard::to_stdvec(self)?;
        Ok(blake3::hash(&bytes))
    }

    /// # Errors
    ///
    /// Returns an error if the record could not be saved.
    pub fn save(&self, doc: &LoroDoc) -> anyhow::Result<()> {
        let map = doc.get_map("record");
        self.reconcile(&map)?;
        Ok(())
    }

    /// # Errors
    ///
    /// Returns an error if the record container is malformed.
    pub fn load(doc: &LoroDoc) -> anyhow::Result<Self> {
        let map = doc.get_map("record");
        let value = map.get_deep_value();
        Self::hydrate(&value).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

#[cfg(test)]
mod tests {
    use loro::LoroDoc;
    use rstest::rstest;
    use xdid::core::did::Did;

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

        assert_eq!(record.creator.to_string(), loaded.creator.to_string());
        assert_eq!(record.nonce, loaded.nonce);
        assert_eq!(record.timestamp, loaded.timestamp);
        assert_eq!(record.schemas.len(), loaded.schemas.len());
    }
}
