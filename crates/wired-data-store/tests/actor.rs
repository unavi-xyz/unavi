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
