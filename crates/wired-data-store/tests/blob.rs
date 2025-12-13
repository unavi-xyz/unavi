mod common;

use std::str::FromStr;

use common::{BLOB_HELLO, BLOB_NONEXISTENT, DID_ALICE, DID_BOB, SCHEMA_TEST, create_test_store};
use wired_data_store::{BlobId, DataStore, Genesis};
use xdid::core::did::Did;

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

#[tokio::test]
async fn test_link_blob_to_record() {
    let (store, _dir) = create_test_store(DID_ALICE).await;

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), SCHEMA_TEST);
    let record_id = store.create_record(genesis).await.expect("create record");
    let blob_id = store.store_blob(BLOB_HELLO).await.expect("store blob");

    store
        .link_blob_to_record(&record_id, &blob_id)
        .await
        .expect("link blob to record");

    assert!(store.get_blob(&blob_id).expect("get blob").is_some());
}

#[tokio::test]
async fn test_link_blob_fails_without_upload() {
    let (store, _dir) = create_test_store(DID_ALICE).await;

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), SCHEMA_TEST);
    let record_id = store.create_record(genesis).await.expect("create record");

    let fake_blob_id = BlobId::from_bytes(b"never uploaded");
    let result = store.link_blob_to_record(&record_id, &fake_blob_id).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_cross_user_blob_isolation() {
    let dir = tempfile::tempdir().expect("create temp dir");

    let store_alice = DataStore::new(
        dir.path().to_path_buf(),
        Did::from_str(DID_ALICE).expect("parse DID"),
    )
    .await
    .expect("create alice store");
    let store_bob = DataStore::new(
        dir.path().to_path_buf(),
        Did::from_str(DID_BOB).expect("parse DID"),
    )
    .await
    .expect("create bob store");

    let blob_id = store_alice
        .store_blob(BLOB_HELLO)
        .await
        .expect("alice stores blob");

    let genesis = Genesis::new(Did::from_str(DID_BOB).expect("parse DID"), SCHEMA_TEST);
    let record_id = store_bob
        .create_record(genesis)
        .await
        .expect("bob creates record");

    let result = store_bob.link_blob_to_record(&record_id, &blob_id).await;
    assert!(
        result.is_err(),
        "bob shouldn't be able to link alice's blob"
    );
}

#[tokio::test]
async fn test_blob_shared_on_disk_but_separate_ownership() {
    let dir = tempfile::tempdir().expect("create temp dir");

    let store_alice = DataStore::new(
        dir.path().to_path_buf(),
        Did::from_str(DID_ALICE).expect("parse DID"),
    )
    .await
    .expect("create alice store");
    let store_bob = DataStore::new(
        dir.path().to_path_buf(),
        Did::from_str(DID_BOB).expect("parse DID"),
    )
    .await
    .expect("create bob store");

    let blob_id_alice = store_alice
        .store_blob(BLOB_HELLO)
        .await
        .expect("alice stores blob");
    let blob_id_bob = store_bob
        .store_blob(BLOB_HELLO)
        .await
        .expect("bob stores blob");

    assert_eq!(blob_id_alice, blob_id_bob, "same content = same CID");

    let genesis_a = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"), SCHEMA_TEST);
    let genesis_b = Genesis::new(Did::from_str(DID_BOB).expect("parse DID"), SCHEMA_TEST);
    let record_a = store_alice
        .create_record(genesis_a)
        .await
        .expect("alice creates record");
    let record_b = store_bob
        .create_record(genesis_b)
        .await
        .expect("bob creates record");

    store_alice
        .link_blob_to_record(&record_a, &blob_id_alice)
        .await
        .expect("alice links blob");
    store_bob
        .link_blob_to_record(&record_b, &blob_id_bob)
        .await
        .expect("bob links blob");

    store_alice
        .pin_record(&record_a, Some(1))
        .await
        .expect("alice pins with expiry");
    store_bob
        .pin_record(&record_b, None)
        .await
        .expect("bob pins permanently");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    store_alice.garbage_collect().await.expect("run GC");

    assert!(
        store_bob
            .get_blob(&blob_id_bob)
            .expect("get blob")
            .is_some(),
        "blob should still exist - bob still has it"
    );
}
