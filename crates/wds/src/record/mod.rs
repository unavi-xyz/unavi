use std::str::FromStr;

use blake3::Hash;
use loro::{LoroDoc, LoroValue};
use rand::Rng;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use xdid::core::did::Did;

pub mod acl;
pub mod envelope;
pub mod schema;
pub(crate) mod validate;

type RecordNonce = [u8; 16];

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub author: Did,
    pub nonce: RecordNonce,
    pub schemas: Vec<Hash>,
    pub timestamp: i64,
}

impl Record {
    #[must_use]
    pub fn new(author: Did) -> Self {
        let mut nonce = RecordNonce::default();
        rand::rng().fill(&mut nonce);

        let schemas = vec![schema::SCHEMA_ACL.hash, schema::SCHEMA_RECORD.hash];

        Self {
            author,
            nonce,
            schemas,
            timestamp: OffsetDateTime::now_utc().unix_timestamp(),
        }
    }

    pub fn add_schema(&mut self, schema: Hash) {
        self.schemas.push(schema);
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

        map.insert("author", self.author.to_string())?;
        map.insert("nonce", &self.nonce)?;

        let schemas = self.schemas.iter().map(Hash::to_string).collect::<Vec<_>>();
        map.insert("schemas", schemas)?;

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

        let author = extract_string(map, "author")?;
        let author = Did::from_str(&author)?;

        let nonce = extract_binary(map, "nonce")?;
        let nonce: RecordNonce = nonce
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid nonce length"))?;

        let timestamp = extract_i64(map, "timestamp")?;
        let schemas = extract_hash_list(map, "schemas")?;

        Ok(Self {
            author,
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

/// Extract a list of blake3 hashes from a map field.
fn extract_hash_list(map: &loro::LoroMapValue, key: &str) -> anyhow::Result<Vec<Hash>> {
    let Some(value) = map.get(key) else {
        return Ok(Vec::new());
    };
    let LoroValue::List(list) = value else {
        anyhow::bail!("{key} is not a list");
    };

    list.iter()
        .map(|v| {
            let LoroValue::String(s) = v else {
                anyhow::bail!("{key} contains non-string value");
            };
            Hash::from_hex(s.as_ref()).map_err(|e| anyhow::anyhow!("invalid hash in {key}: {e}"))
        })
        .collect()
}
