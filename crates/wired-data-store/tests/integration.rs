use std::str::FromStr;

use wired_data_store::{BlobId, DataStore, Genesis, RecordId};
use xdid::core::did::Did;

const DID_ALICE: &str = "did:example:alice";
const DID_BOB: &str = "did:example:bob";
const DID_CHARLIE: &str = "did:example:charlie";
const DID_DAVE: &str = "did:example:dave";
const DID_EVE: &str = "did:example:eve";
const DID_FRANK: &str = "did:example:frank";
const DID_NOBODY: &str = "did:example:nobody";

const SCHEMA_TEST: &str = "test-schema";
const SCHEMA_NONCE_TEST: &str = "nonce-test";
const SCHEMA_DELETE_TEST: &str = "delete-test";
const SCHEMA_PIN_TEST: &str = "pin-test";
const SCHEMA_A: &str = "schema-a";
const SCHEMA_B: &str = "schema-b";
const SCHEMA_FAKE: &str = "fake";

const BLOB_HELLO: &[u8] = b"hello, world!";
const BLOB_NONEXISTENT: &[u8] = b"nonexistent data";

async fn create_test_store(owner_did: &str) -> (DataStore, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let did = Did::from_str(owner_did).expect("parse DID");
    let store = DataStore::new(dir.path().to_path_buf(), did)
        .await
        .expect("create data store");
    (store, dir)
}

#[tokio::test]
async fn test_create_and_get_record() {
    let (store, _dir) = create_test_store(DID_ALICE).await;

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), SCHEMA_TEST);
    let id = store.create_record(genesis).await.expect("create record");

    let record = store
        .get_record(&id)
        .await
        .expect("get record")
        .expect("record exists");

    assert_eq!(record.id, id);
    assert_eq!(record.genesis.creator.to_string(), DID_ALICE);
    assert_eq!(record.genesis.schema.as_str(), SCHEMA_TEST);
}

#[tokio::test]
async fn test_record_preserves_nonce() {
    let (store, _dir) = create_test_store(DID_BOB).await;

    let genesis = Genesis::new(
        Did::from_str(DID_BOB).expect("parse DID"),
        SCHEMA_NONCE_TEST,
    );
    let original_nonce = genesis.nonce;
    let id = store.create_record(genesis).await.expect("create record");

    let record = store
        .get_record(&id)
        .await
        .expect("get record")
        .expect("record exists");

    assert_eq!(record.genesis.nonce, original_nonce);
}

#[tokio::test]
async fn test_get_nonexistent_record() {
    let (store, _dir) = create_test_store(DID_NOBODY).await;

    let genesis = Genesis::new(Did::from_str(DID_NOBODY).expect("parse DID"), SCHEMA_FAKE);
    let fake_id = RecordId(genesis.cid());

    let result = store.get_record(&fake_id).await.expect("query succeeds");
    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_record() {
    let (store, _dir) = create_test_store(DID_CHARLIE).await;

    let genesis = Genesis::new(
        Did::from_str(DID_CHARLIE).expect("parse DID"),
        SCHEMA_DELETE_TEST,
    );
    let id = store.create_record(genesis).await.expect("create record");

    assert!(store.get_record(&id).await.expect("get record").is_some());

    store.delete_record(&id).await.expect("delete record");

    assert!(store.get_record(&id).await.expect("get record").is_none());
}

#[tokio::test]
async fn test_store_and_get_blob() {
    let (store, _dir) = create_test_store(DID_ALICE).await;

    let id = store.store_blob(BLOB_HELLO).await.expect("store blob");

    let retrieved = store.get_blob(&id).expect("get blob").expect("blob exists");

    assert_eq!(retrieved, BLOB_HELLO);
}

#[tokio::test]
async fn test_blob_content_addressing() {
    let (store, _dir) = create_test_store(DID_ALICE).await;

    let id1 = store.store_blob(BLOB_HELLO).await.expect("store blob 1");
    let id2 = store.store_blob(BLOB_HELLO).await.expect("store blob 2");

    assert_eq!(id1, id2);
}

#[tokio::test]
async fn test_get_nonexistent_blob() {
    let (store, _dir) = create_test_store(DID_ALICE).await;

    let fake_id = BlobId::from_bytes(BLOB_NONEXISTENT);
    let result = store.get_blob(&fake_id).expect("query succeeds");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_pin_and_unpin_record() {
    let (store, _dir) = create_test_store(DID_DAVE).await;

    let genesis = Genesis::new(Did::from_str(DID_DAVE).expect("parse DID"), SCHEMA_PIN_TEST);
    let record_id = store.create_record(genesis).await.expect("create record");

    store
        .pin_record(&record_id, None)
        .await
        .expect("pin record");

    store.unpin_record(&record_id).await.expect("unpin record");
}

#[tokio::test]
async fn test_pin_with_ttl() {
    let (store, _dir) = create_test_store(DID_DAVE).await;

    let genesis = Genesis::new(Did::from_str(DID_DAVE).expect("parse DID"), SCHEMA_PIN_TEST);
    let record_id = store.create_record(genesis).await.expect("create record");

    store
        .pin_record(&record_id, Some(3600))
        .await
        .expect("pin record with TTL");

    store.unpin_record(&record_id).await.expect("unpin record");
}

#[tokio::test]
async fn test_multiple_records() {
    let (store, _dir) = create_test_store(DID_EVE).await;

    let genesis1 = Genesis::new(Did::from_str(DID_EVE).expect("parse DID"), SCHEMA_A);
    let genesis2 = Genesis::new(Did::from_str(DID_FRANK).expect("parse DID"), SCHEMA_B);

    let id1 = store
        .create_record(genesis1)
        .await
        .expect("create record 1");
    let id2 = store
        .create_record(genesis2)
        .await
        .expect("create record 2");

    assert_ne!(id1, id2);

    let record1 = store
        .get_record(&id1)
        .await
        .expect("get record 1")
        .expect("record 1 exists");

    let record2 = store
        .get_record(&id2)
        .await
        .expect("get record 2")
        .expect("record 2 exists");

    assert_eq!(record1.genesis.creator.to_string(), DID_EVE);
    assert_eq!(record2.genesis.creator.to_string(), DID_FRANK);
}

#[tokio::test]
async fn test_did_isolation() {
    let dir = tempfile::tempdir().expect("create temp dir");

    let store1 = DataStore::new(
        dir.path().to_path_buf(),
        Did::from_str(DID_ALICE).expect("parse DID"),
    )
    .await
    .expect("create store 1");
    let store2 = DataStore::new(
        dir.path().to_path_buf(),
        Did::from_str(DID_BOB).expect("parse DID"),
    )
    .await
    .expect("create store 2");

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), SCHEMA_TEST);
    let id = store1
        .create_record(genesis)
        .await
        .expect("alice creates record");

    assert!(
        store2
            .get_record(&id)
            .await
            .expect("query succeeds")
            .is_none(),
        "bob should not see alice's record"
    );
}

#[tokio::test]
async fn test_shared_blob_deduplication() {
    let dir = tempfile::tempdir().expect("create temp dir");

    let store1 = DataStore::new(
        dir.path().to_path_buf(),
        Did::from_str(DID_ALICE).expect("parse DID"),
    )
    .await
    .expect("create store 1");
    let store2 = DataStore::new(
        dir.path().to_path_buf(),
        Did::from_str(DID_BOB).expect("parse DID"),
    )
    .await
    .expect("create store 2");

    let id1 = store1
        .store_blob(BLOB_HELLO)
        .await
        .expect("alice stores blob");
    let id2 = store2
        .store_blob(BLOB_HELLO)
        .await
        .expect("bob stores blob");

    assert_eq!(id1, id2, "same content should have same CID");

    assert_eq!(
        store1
            .get_blob(&id1)
            .expect("alice retrieves blob")
            .expect("blob exists"),
        BLOB_HELLO
    );
    assert_eq!(
        store2
            .get_blob(&id2)
            .expect("bob retrieves blob")
            .expect("blob exists"),
        BLOB_HELLO
    );

    let blob_path = dir
        .path()
        .join("blobs")
        .join(&id1.as_str()[..2])
        .join(id1.as_str());
    assert!(blob_path.exists(), "only one copy should exist on disk");
}
