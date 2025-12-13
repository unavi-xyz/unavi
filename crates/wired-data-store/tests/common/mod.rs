#![allow(dead_code)]

use std::str::FromStr;

use wired_data_store::DataStore;
use xdid::core::did::Did;

pub const DID_ALICE: &str = "did:example:alice";
pub const DID_BOB: &str = "did:example:bob";
pub const DID_CHARLIE: &str = "did:example:charlie";
pub const DID_DAVE: &str = "did:example:dave";
pub const DID_EVE: &str = "did:example:eve";
pub const DID_FRANK: &str = "did:example:frank";
pub const DID_NOBODY: &str = "did:example:nobody";

pub const SCHEMA_TEST: &str = "test-schema";
pub const SCHEMA_NONCE_TEST: &str = "nonce-test";
pub const SCHEMA_DELETE_TEST: &str = "delete-test";
pub const SCHEMA_PIN_TEST: &str = "pin-test";
pub const SCHEMA_A: &str = "schema-a";
pub const SCHEMA_B: &str = "schema-b";
pub const SCHEMA_FAKE: &str = "fake";

pub const BLOB_HELLO: &[u8] = b"hello, world!";
pub const BLOB_NONEXISTENT: &[u8] = b"nonexistent data";

pub async fn create_test_store(owner_did: &str) -> (DataStore, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let did = Did::from_str(owner_did).expect("parse DID");
    let store = DataStore::new(dir.path().to_path_buf(), did)
        .await
        .expect("create data store");
    (store, dir)
}
