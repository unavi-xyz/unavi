mod common;

use std::str::FromStr;

use common::{
    DID_ALICE, DID_BOB, DID_CHARLIE, DID_EVE, DID_FRANK, DID_NOBODY, SCHEMA_A, SCHEMA_B,
    SCHEMA_DELETE_TEST, SCHEMA_FAKE, SCHEMA_NONCE_TEST, SCHEMA_TEST, create_test_store,
};
use wired_data_store::{DataStore, Genesis, RecordId};
use xdid::core::did::Did;

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
