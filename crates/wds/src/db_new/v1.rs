use native_db::{ToKey, native_db};
use native_model::{Model, native_model, postcard_1_0::PostCard};
use serde::{Deserialize, Serialize};

use super::keys::{BoolKey, DidKey, HashKey};

#[derive(Serialize, Deserialize, Debug)]
#[native_db]
#[native_model(id = 1, version = 1, with = PostCard)]
pub struct Record {
    #[primary_key]
    pub id: HashKey,
    #[secondary_key]
    pub creator: DidKey,
    #[secondary_key]
    pub is_public: BoolKey,
    pub nonce: [u8; 16],
    pub size: u64,
    pub timestamp: i64,
    pub vv: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepType {
    Schema,
}

#[derive(Serialize, Deserialize, Debug)]
#[native_db]
#[native_model(id = 2, version = 1, with = PostCard)]
pub struct RecordDep {
    #[primary_key]
    pub key: (HashKey, HashKey), // (record_id, blob_hash)
    #[secondary_key]
    pub record_id: HashKey,
    #[secondary_key]
    pub blob_hash: HashKey,
    pub dep_type: DepType,
}

#[derive(Serialize, Deserialize, Debug)]
#[native_db]
#[native_model(id = 3, version = 1, with = PostCard)]
pub struct RecordSchema {
    #[primary_key]
    pub key: (HashKey, HashKey), // (record_id, schema_hash)
    #[secondary_key]
    pub schema_hash: HashKey,
}

#[derive(Serialize, Deserialize, Debug)]
#[native_db]
#[native_model(id = 4, version = 1, with = PostCard)]
pub struct RecordAclRead {
    #[primary_key]
    pub key: (HashKey, DidKey), // (record_id, did)
    #[secondary_key]
    pub did: DidKey,
}

#[derive(Serialize, Deserialize, Debug)]
#[native_db]
#[native_model(id = 5, version = 1, with = PostCard)]
pub struct Envelope {
    #[primary_key]
    pub id: u64,
    #[secondary_key]
    pub record_id: HashKey,
    pub author: DidKey,
    pub from_vv: Vec<u8>,
    pub to_vv: Vec<u8>,
    pub ops: Vec<u8>,
    pub payload_bytes: Vec<u8>,
    pub signature: Vec<u8>,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[native_db]
#[native_model(id = 6, version = 1, with = PostCard)]
pub struct BlobPin {
    #[primary_key]
    pub key: (DidKey, HashKey), // (owner, hash)
    #[secondary_key]
    pub hash: HashKey,
    #[secondary_key]
    pub owner: DidKey,
    pub expires: i64,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[native_db]
#[native_model(id = 7, version = 1, with = PostCard)]
pub struct RecordPin {
    #[primary_key]
    pub key: (DidKey, HashKey), // (owner, record_id)
    #[secondary_key]
    pub record_id: HashKey,
    #[secondary_key]
    pub owner: DidKey,
    pub expires: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[native_db]
#[native_model(id = 8, version = 1, with = PostCard)]
pub struct UserQuota {
    #[primary_key]
    pub owner: DidKey,
    pub bytes_used: u64,
    pub quota_bytes: u64,
}
