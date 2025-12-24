use blake3::Hash;
use rand::Rng;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use xdid::core::did::Did;

mod schema;

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

    pub fn id(&self) -> postcard::Result<Hash> {
        let bytes = postcard::to_stdvec(self)?;
        Ok(blake3::hash(&bytes))
    }
}
