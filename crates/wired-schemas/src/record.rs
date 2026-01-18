//! Record type for WDS documents.

use std::collections::BTreeMap;

use blake3::Hash;
use loro::LoroDoc;
use loro_surgeon::{Hydrate, HydrateError, Reconcile, ReconcileError, loro::LoroMap};
use rand::Rng;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use time::OffsetDateTime;
use xdid::core::did::Did;

use crate::{conv, schemas};

/// Fixed-size nonce for record identification.
pub type RecordNonce = [u8; 16];

/// A WDS record containing metadata about the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    /// The DID of the record creator.
    pub creator: Did,
    /// Random nonce for unique record identification.
    pub nonce: RecordNonce,
    /// Map of container names to their schema hashes.
    pub schemas: BTreeMap<SmolStr, Hash>,
    /// Unix timestamp of record creation.
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

    /// Add a schema for a container.
    pub fn add_schema(&mut self, container: SmolStr, schema: Hash) {
        self.schemas.insert(container, schema);
    }

    /// Compute the content-addressed ID of this record.
    ///
    /// # Errors
    ///
    /// Returns an error if the record could not be serialized.
    pub fn id(&self) -> postcard::Result<Hash> {
        let bytes = postcard::to_stdvec(self)?;
        Ok(blake3::hash(&bytes))
    }

    /// Save record to a [`LoroDoc`]'s "record" container.
    ///
    /// # Errors
    ///
    /// Returns an error if the record could not be saved.
    pub fn save(&self, doc: &LoroDoc) -> anyhow::Result<()> {
        let map = doc.get_map("record");
        self.reconcile(&map)?;
        Ok(())
    }

    /// Load record from a [`LoroDoc`]'s "record" container.
    ///
    /// # Errors
    ///
    /// Returns an error if the record container is malformed.
    pub fn load(doc: &LoroDoc) -> anyhow::Result<Self> {
        let map = doc.get_map("record");
        let value = map.get_deep_value();
        Self::hydrate(&value).map_err(|e| anyhow::anyhow!("{e}"))
    }
}

impl Hydrate for Record {
    fn hydrate(value: &loro::LoroValue) -> Result<Self, HydrateError> {
        let loro::LoroValue::Map(map) = value else {
            return Err(HydrateError::TypeMismatch {
                path: String::new(),
                expected: "map".into(),
                actual: format!("{value:?}"),
            });
        };

        let creator = map
            .get("creator")
            .ok_or_else(|| HydrateError::MissingField("creator".into()))?;
        let creator = conv::did::hydrate(creator)?;

        let nonce = map
            .get("nonce")
            .ok_or_else(|| HydrateError::MissingField("nonce".into()))?;
        let loro::LoroValue::Binary(nonce_bytes) = nonce else {
            return Err(HydrateError::TypeMismatch {
                path: "nonce".into(),
                expected: "binary".into(),
                actual: format!("{nonce:?}"),
            });
        };
        let nonce: RecordNonce = nonce_bytes
            .to_vec()
            .try_into()
            .map_err(|_| HydrateError::Custom("invalid nonce length".into()))?;

        let schemas = map
            .get("schemas")
            .ok_or_else(|| HydrateError::MissingField("schemas".into()))?;
        let schemas = conv::hash::map::hydrate(schemas)?;

        let timestamp = map
            .get("timestamp")
            .ok_or_else(|| HydrateError::MissingField("timestamp".into()))?;
        let loro::LoroValue::I64(timestamp) = timestamp else {
            return Err(HydrateError::TypeMismatch {
                path: "timestamp".into(),
                expected: "i64".into(),
                actual: format!("{timestamp:?}"),
            });
        };

        Ok(Self {
            creator,
            nonce,
            schemas,
            timestamp: *timestamp,
        })
    }
}

impl Reconcile for Record {
    fn reconcile(&self, map: &LoroMap) -> Result<(), ReconcileError> {
        conv::did::reconcile_field(&self.creator, map, "creator")?;
        map.insert("nonce", self.nonce.as_slice())?;
        conv::hash::map::reconcile_field(&self.schemas, map, "schemas")?;
        map.insert("timestamp", self.timestamp)?;
        Ok(())
    }

    fn reconcile_field(&self, map: &LoroMap, key: &str) -> Result<(), ReconcileError> {
        let nested = map.get_or_create_container(key, loro::LoroMap::new())?;
        self.reconcile(&nested)
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
