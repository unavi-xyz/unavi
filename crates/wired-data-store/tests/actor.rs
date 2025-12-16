mod common;

use wired_data_store::{Actor, DataStore, ValidatedView};
use xdid::methods::key::{DidKeyPair, PublicKey, p256::P256KeyPair};

async fn create_actor() -> (Actor, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("create temp dir");
    let store = DataStore::new(dir.path().to_path_buf())
        .await
        .expect("create store");

    let signing_key = P256KeyPair::generate();
    let did = signing_key.public().to_did();

    let view = store.view_for_user(did.clone());
    let validated_view = ValidatedView::new(view).expect("create validated view");

    let actor = Actor::new(did, signing_key, validated_view);
    (actor, dir)
}

#[tokio::test]
async fn test_actor_create_record() {
    let (actor, _dir) = create_actor().await;

    let record_id = actor.create_record(Some("test-schema")).await;
    assert!(record_id.is_ok(), "should create record: {:?}", record_id);

    let record_id = record_id.expect("record id");

    // Verify record exists.
    let record = actor.view().inner().get_record(&record_id).await;
    assert!(record.is_ok());
    let record = record.expect("get record").expect("record should exist");
    assert_eq!(record.genesis.creator, *actor.did());
}

#[tokio::test]
async fn test_actor_update_record() {
    let (actor, _dir) = create_actor().await;

    // Create a record.
    let record_id = actor
        .create_record(Some("test-schema"))
        .await
        .expect("create record");

    // Update the record.
    let result = actor
        .update_record(&record_id, |doc| {
            let map = doc.get_map("test");
            map.insert("key", "value")?;
            Ok(())
        })
        .await;

    assert!(result.is_ok(), "should update record: {:?}", result);

    // Verify the update was applied.
    let record = actor
        .view()
        .inner()
        .get_record(&record_id)
        .await
        .expect("get record")
        .expect("record should exist");

    let map = record.doc().get_map("test");
    let value = map.get("key");
    assert!(value.is_some(), "key should exist in map");
}

#[tokio::test]
async fn test_actor_noop_update() {
    let (actor, _dir) = create_actor().await;

    // Create a record.
    let record_id = actor
        .create_record(Some("test-schema"))
        .await
        .expect("create record");

    // Update with no changes should succeed (no-op).
    let result = actor.update_record(&record_id, |_doc| Ok(())).await;

    assert!(result.is_ok(), "noop update should succeed: {:?}", result);
}

#[tokio::test]
async fn test_actor_create_snapshot() {
    let (actor, _dir) = create_actor().await;

    // Create a record.
    let record_id = actor
        .create_record(Some("test-schema"))
        .await
        .expect("create record");

    // Make some updates.
    actor
        .update_record(&record_id, |doc| {
            let map = doc.get_map("test");
            map.insert("key1", "value1")?;
            Ok(())
        })
        .await
        .expect("update 1");

    actor
        .update_record(&record_id, |doc| {
            let map = doc.get_map("test");
            map.insert("key2", "value2")?;
            Ok(())
        })
        .await
        .expect("update 2");

    // Create a snapshot.
    let snapshot = actor.create_snapshot(&record_id).await;
    assert!(snapshot.is_ok(), "should create snapshot: {:?}", snapshot);

    let snapshot = snapshot.expect("snapshot");
    assert_eq!(snapshot.snapshot_num, 1);
    assert_eq!(snapshot.record_id, record_id);
    assert!(!snapshot.loro_snapshot.is_empty());
    assert!(!snapshot.signature.bytes.is_empty());
}

#[tokio::test]
async fn test_multiple_snapshots() {
    let (actor, _dir) = create_actor().await;

    // Create a record.
    let record_id = actor
        .create_record(Some("test-schema"))
        .await
        .expect("create record");

    // Create first snapshot.
    actor
        .update_record(&record_id, |doc| {
            let map = doc.get_map("test");
            map.insert("key1", "value1")?;
            Ok(())
        })
        .await
        .expect("update");
    let snap1 = actor.create_snapshot(&record_id).await.expect("snapshot 1");
    assert_eq!(snap1.snapshot_num, 1);

    // Create second snapshot.
    actor
        .update_record(&record_id, |doc| {
            let map = doc.get_map("test");
            map.insert("key2", "value2")?;
            Ok(())
        })
        .await
        .expect("update");
    let snap2 = actor.create_snapshot(&record_id).await.expect("snapshot 2");
    assert_eq!(snap2.snapshot_num, 2);

    // Create third snapshot (this should trigger GC of snap1).
    actor
        .update_record(&record_id, |doc| {
            let map = doc.get_map("test");
            map.insert("key3", "value3")?;
            Ok(())
        })
        .await
        .expect("update");
    let snap3 = actor.create_snapshot(&record_id).await.expect("snapshot 3");
    assert_eq!(snap3.snapshot_num, 3);
}

#[tokio::test]
async fn test_snapshot_preserves_data() {
    let (actor, _dir) = create_actor().await;

    // Create a record.
    let record_id = actor
        .create_record(Some("test-schema"))
        .await
        .expect("create record");

    // Make updates.
    actor
        .update_record(&record_id, |doc| {
            let map = doc.get_map("data");
            map.insert("name", "Test Record")?;
            map.insert("count", 42)?;
            Ok(())
        })
        .await
        .expect("update");

    // Create a snapshot.
    let snapshot = actor.create_snapshot(&record_id).await.expect("snapshot");

    // Verify snapshot contains the data.
    assert!(!snapshot.loro_snapshot.is_empty());
    assert!(!snapshot.genesis_bytes.is_empty());
    assert!(!snapshot.version.is_empty());

    // Verify record still has the data after snapshot.
    let record = actor
        .view()
        .inner()
        .get_record(&record_id)
        .await
        .expect("get record")
        .expect("record exists");

    let map = record.doc().get_map("data");
    assert!(map.get("name").is_some());
    assert!(map.get("count").is_some());
}
