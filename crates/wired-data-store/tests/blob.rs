mod common;

use std::str::FromStr;

use common::{
    BLOB_HELLO, BLOB_NONEXISTENT, DID_ALICE, DID_BOB, create_test_store, create_test_view,
};
use wired_data_store::{BlobId, Genesis};
use xdid::core::did::Did;

#[tokio::test]
async fn test_store_and_get_blob() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    let id = view.store_blob(BLOB_HELLO).await.expect("store blob");

    let retrieved = view
        .get_blob(&id)
        .await
        .expect("get blob")
        .expect("blob exists");

    assert_eq!(retrieved, BLOB_HELLO);
}

#[tokio::test]
async fn test_blob_content_addressing() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    let id1 = view.store_blob(BLOB_HELLO).await.expect("store blob 1");
    let id2 = view.store_blob(BLOB_HELLO).await.expect("store blob 2");

    assert_eq!(id1, id2);
}

#[tokio::test]
async fn test_get_nonexistent_blob() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    let fake_id = BlobId::from_bytes(BLOB_NONEXISTENT);
    let result = view.get_blob(&fake_id).await.expect("query succeeds");

    assert!(result.is_none());
}

#[tokio::test]
async fn test_shared_blob_deduplication() {
    let (store, _dir) = create_test_store().await;

    let view1 = store.view_for_user(Did::from_str(DID_ALICE).expect("parse DID"));
    let view2 = store.view_for_user(Did::from_str(DID_BOB).expect("parse DID"));

    let id1 = view1
        .store_blob(BLOB_HELLO)
        .await
        .expect("alice stores blob");
    let id2 = view2.store_blob(BLOB_HELLO).await.expect("bob stores blob");

    assert_eq!(id1, id2, "same content should have same id");

    assert_eq!(
        view1
            .get_blob(&id1)
            .await
            .expect("alice retrieves blob")
            .expect("blob exists"),
        BLOB_HELLO
    );
    assert_eq!(
        view2
            .get_blob(&id2)
            .await
            .expect("bob retrieves blob")
            .expect("blob exists"),
        BLOB_HELLO
    );

    // Note: FsStore uses its own directory structure, so we don't check the exact path.
    // The deduplication is handled internally by FsStore.
}

#[tokio::test]
async fn test_link_blob_to_record() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"));
    let record_id = view.create_record(genesis).await.expect("create record");
    let blob_id = view.store_blob(BLOB_HELLO).await.expect("store blob");

    view.link_blob_to_record(record_id, &blob_id)
        .await
        .expect("link blob to record");

    assert!(view.get_blob(&blob_id).await.expect("get blob").is_some());
}

#[tokio::test]
async fn test_link_blob_fails_without_upload() {
    let (view, _dir) = create_test_view(DID_ALICE).await;

    let genesis = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"));
    let record_id = view.create_record(genesis).await.expect("create record");

    let fake_blob_id = BlobId::from_bytes(b"never uploaded");
    let result = view.link_blob_to_record(record_id, &fake_blob_id).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_cross_user_blob_isolation() {
    let (store, _dir) = create_test_store().await;

    let view_alice = store.view_for_user(Did::from_str(DID_ALICE).expect("parse DID"));
    let view_bob = store.view_for_user(Did::from_str(DID_BOB).expect("parse DID"));

    let blob_id = view_alice
        .store_blob(BLOB_HELLO)
        .await
        .expect("alice stores blob");

    let genesis = Genesis::new(Did::from_str(DID_BOB).expect("parse DID"));
    let record_id = view_bob
        .create_record(genesis)
        .await
        .expect("bob creates record");

    let result = view_bob.link_blob_to_record(record_id, &blob_id).await;
    assert!(
        result.is_err(),
        "bob shouldn't be able to link alice's blob"
    );
}

#[tokio::test]
async fn test_blob_shared_on_disk_but_separate_ownership() {
    let (store, _dir) = create_test_store().await;

    let view_alice = store.view_for_user(Did::from_str(DID_ALICE).expect("parse DID"));
    let view_bob = store.view_for_user(Did::from_str(DID_BOB).expect("parse DID"));

    let blob_id_alice = view_alice
        .store_blob(BLOB_HELLO)
        .await
        .expect("alice stores blob");
    let blob_id_bob = view_bob
        .store_blob(BLOB_HELLO)
        .await
        .expect("bob stores blob");

    assert_eq!(blob_id_alice, blob_id_bob, "same content = same ID");

    let genesis_a = Genesis::new(Did::from_str(DID_ALICE).expect("parse DID"));
    let genesis_b = Genesis::new(Did::from_str(DID_BOB).expect("parse DID"));
    let record_a = view_alice
        .create_record(genesis_a)
        .await
        .expect("alice creates record");
    let record_b = view_bob
        .create_record(genesis_b)
        .await
        .expect("bob creates record");

    view_alice
        .link_blob_to_record(record_a, &blob_id_alice)
        .await
        .expect("alice links blob");
    view_bob
        .link_blob_to_record(record_b, &blob_id_bob)
        .await
        .expect("bob links blob");

    view_alice
        .pin_record(record_a, Some(1))
        .await
        .expect("alice pins with expiry");
    view_bob
        .pin_record(record_b, None)
        .await
        .expect("bob pins permanently");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    store.garbage_collect().await.expect("run GC");

    assert!(
        view_bob
            .get_blob(&blob_id_bob)
            .await
            .expect("get blob")
            .is_some(),
        "blob should still exist - bob still has it"
    );
}
