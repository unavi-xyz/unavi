use blake3::Hash;
use loro::LoroDoc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use xdid::core::did::Did;

pub mod acl;
pub mod envelope;
pub mod schema;
pub mod validate;

type RecordNonce = [u8; 16];

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    author: Did,
    nonce: RecordNonce,
    schemas: Vec<Hash>,
    timestamp: i64,
}

impl Record {
    pub fn new(author: Did) -> Self {
        let mut nonce = RecordNonce::default();
        rand::rng().fill(&mut nonce);

        let schemas = vec![*schema::SCHEMA_ACL, *schema::SCHEMA_RECORD];

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

    pub fn id(&self) -> postcard::Result<Hash> {
        let bytes = postcard::to_stdvec(self)?;
        Ok(blake3::hash(&bytes))
    }

    pub fn save(&self, doc: &LoroDoc) -> anyhow::Result<()> {
        let map = doc.get_map("record");

        map.insert("author", self.author.to_string())?;
        map.insert("nonce", &self.nonce)?;

        let schemas = self.schemas.iter().map(Hash::to_string).collect::<Vec<_>>();
        map.insert("schemas", schemas)?;

        map.insert("timestamp", self.timestamp)?;

        Ok(())
    }
}
