use std::{collections::BTreeMap, str::FromStr};

use blake3::Hash;
use loro::{LoroDoc, LoroValue};
use rand::Rng;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use time::OffsetDateTime;
use xdid::core::did::Did;

pub mod acl;
pub mod envelope;
pub mod schema;
pub(crate) mod validate;

type RecordNonce = [u8; 16];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub creator: Did,
    pub nonce: RecordNonce,
    pub schemas: BTreeMap<SmolStr, Hash>,
    pub timestamp: i64,
}

impl Record {
    #[must_use]
    pub fn new(creator: Did) -> Self {
        let mut nonce = RecordNonce::default();
        rand::rng().fill(&mut nonce);

        let mut schemas = BTreeMap::new();
        schemas.insert("acl".into(), schema::SCHEMA_ACL.hash);
        schemas.insert("record".into(), schema::SCHEMA_RECORD.hash);

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

        map.insert("creator", self.creator.to_string())?;
        map.insert("nonce", &self.nonce)?;

        // Save schemas as map: {"container_name": "schema_hash_hex"}.
        let schemas_map = doc.get_map("record_schemas_temp");
        for (container, hash) in &self.schemas {
            schemas_map.insert(container.as_str(), hash.to_string())?;
        }
        let schemas_value = schemas_map.get_deep_value();
        map.insert("schemas", schemas_value)?;

        map.insert("timestamp", self.timestamp)?;

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
        Self::load_from_value(&value)
    }

    /// Load record from an already-extracted [`LoroValue`].
    ///
    /// # Errors
    ///
    /// Returns an error if the value is malformed.
    pub fn load_from_value(value: &LoroValue) -> anyhow::Result<Self> {
        let LoroValue::Map(map) = value else {
            anyhow::bail!("record is not a map");
        };

        let creator = extract_string(map, "creator")?;
        let creator = Did::from_str(&creator)?;

        let nonce = extract_binary(map, "nonce")?;
        let nonce: RecordNonce = nonce
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid nonce length"))?;

        let timestamp = extract_i64(map, "timestamp")?;
        let schemas = extract_schema_map(map, "schemas")?;

        Ok(Self {
            creator,
            nonce,
            schemas,
            timestamp,
        })
    }
}

/// Extract a string field from a map.
fn extract_string(map: &loro::LoroMapValue, key: &str) -> anyhow::Result<String> {
    let Some(value) = map.get(key) else {
        anyhow::bail!("missing field: {key}");
    };
    let LoroValue::String(s) = value else {
        anyhow::bail!("{key} is not a string");
    };
    Ok(s.to_string())
}

/// Extract a binary field from a map.
fn extract_binary(map: &loro::LoroMapValue, key: &str) -> anyhow::Result<Vec<u8>> {
    let Some(value) = map.get(key) else {
        anyhow::bail!("missing field: {key}");
    };
    let LoroValue::Binary(b) = value else {
        anyhow::bail!("{key} is not binary");
    };
    Ok(b.to_vec())
}

/// Extract an i64 field from a map.
fn extract_i64(map: &loro::LoroMapValue, key: &str) -> anyhow::Result<i64> {
    let Some(value) = map.get(key) else {
        anyhow::bail!("missing field: {key}");
    };
    let LoroValue::I64(n) = value else {
        anyhow::bail!("{key} is not an i64");
    };
    Ok(*n)
}

/// Extract a map of container names to schema hashes.
fn extract_schema_map(
    map: &loro::LoroMapValue,
    key: &str,
) -> anyhow::Result<BTreeMap<SmolStr, Hash>> {
    let Some(value) = map.get(key) else {
        return Ok(BTreeMap::new());
    };
    let LoroValue::Map(schema_map) = value else {
        anyhow::bail!("{key} is not a map");
    };

    schema_map
        .iter()
        .map(|(container, v)| {
            let LoroValue::String(hash_str) = v else {
                anyhow::bail!("{key}.{container} is not a string");
            };
            let hash = Hash::from_hex(hash_str.as_ref())
                .map_err(|e| anyhow::anyhow!("invalid hash in {key}.{container}: {e}"))?;
            Ok((container.into(), hash))
        })
        .collect()
}
