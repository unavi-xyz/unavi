use std::time::Duration;

use rstest::rstest;
use tracing_test::traced_test;
use wired_schemas::{Acl, SCHEMA_BEACON, SCHEMA_HOME};

use crate::common::{DataStoreCtx, ctx};

mod common;

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_query_own_records(#[future] ctx: DataStoreCtx) {
    // Alice creates a record.
    let result = ctx.alice.create_record().send().await.expect("create");
    let record_id = result.id;

    // Alice queries with no filters.
    let results = ctx.alice.query().send().await.expect("query");

    assert!(results.contains(&record_id));
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_query_by_creator(#[future] ctx: DataStoreCtx) {
    // Alice and Bob each create a record.
    let alice_result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("alice create");
    let bob_result = ctx.bob.create_record().send().await.expect("bob create");

    // Alice queries for her own records.
    let results = ctx
        .alice
        .query()
        .creator(ctx.alice.identity().did())
        .send()
        .await
        .expect("query");

    assert!(results.contains(&alice_result.id));
    assert!(!results.contains(&bob_result.id));
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_query_by_schema(#[future] ctx: DataStoreCtx) {
    // Alice creates a record with SCHEMA_HOME.
    let home_result = ctx
        .alice
        .create_record()
        .add_schema("home", &*SCHEMA_HOME, |_| Ok(()))
        .expect("add schema")
        .send()
        .await
        .expect("create home");

    // Alice creates a record with SCHEMA_BEACON.
    let beacon_result = ctx
        .alice
        .create_record()
        .add_schema("beacon", &*SCHEMA_BEACON, |_| Ok(()))
        .expect("add schema")
        .send()
        .await
        .expect("create beacon");

    // Query for HOME schema.
    let results = ctx
        .alice
        .query()
        .schema(SCHEMA_HOME.hash)
        .send()
        .await
        .expect("query");

    assert!(results.contains(&home_result.id));
    assert!(!results.contains(&beacon_result.id));
    assert_eq!(results.len(), 1);
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_query_respects_acl(#[future] ctx: DataStoreCtx) {
    // Alice creates a record (only she has access).
    let result = ctx.alice.create_record().send().await.expect("create");

    // Bob queries - should get empty results.
    let results = ctx.bob.query().send().await.expect("query");

    assert!(!results.contains(&result.id));
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_query_with_read_permission(#[future] ctx: DataStoreCtx) {
    // Alice creates a record.
    let result = ctx.alice.create_record().send().await.expect("create");
    let (record_id, doc) = (result.id, result.doc);

    // Alice adds Bob to read ACL.
    let from_vv = doc.oplog_vv();
    let mut acl = Acl::load(&doc).expect("load acl");
    acl.read.push(ctx.bob.identity().did().clone());
    acl.save(&doc).expect("save acl");

    ctx.alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("update acl");

    // Bob queries - should now see Alice's record.
    let results = ctx.bob.query().send().await.expect("query");

    assert!(results.contains(&record_id));
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_query_empty_results(#[future] ctx: DataStoreCtx) {
    // Alice queries for a non-existent schema.
    let fake_hash = blake3::hash(b"fake schema");
    let results = ctx
        .alice
        .query()
        .schema(fake_hash)
        .send()
        .await
        .expect("query");

    assert!(results.is_empty());
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_query_public_record(#[future] ctx: DataStoreCtx) {
    // Alice creates a public record.
    let result = ctx
        .alice
        .create_record()
        .public()
        .send()
        .await
        .expect("create public record");

    // Bob queries - should see Alice's public record.
    let results = ctx.bob.query().send().await.expect("query");

    assert!(results.contains(&result.id));
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_read_public_record(#[future] ctx: DataStoreCtx) {
    // Alice creates a public record with some data.
    let result = ctx
        .alice
        .create_record()
        .public()
        .send()
        .await
        .expect("create public record");
    let (record_id, doc) = (result.id, result.doc);

    // Add some data.
    let from_vv = doc.oplog_vv();
    doc.get_map("data")
        .insert("key", "public_value")
        .expect("insert");
    ctx.alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("update");

    // Bob reads the public record.
    let read_doc = ctx.bob.read(record_id).send().await.expect("read");

    // Verify data is present.
    let value = read_doc.get_map("data").get_deep_value();
    let loro::LoroValue::Map(map) = value else {
        panic!("expected map");
    };
    assert_eq!(
        map.get("key"),
        Some(&loro::LoroValue::String("public_value".into()))
    );
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_private_record_not_queryable_by_others(#[future] ctx: DataStoreCtx) {
    // Alice creates a private record (default).
    let result = ctx.alice.create_record().send().await.expect("create");

    // Bob queries - should NOT see Alice's private record.
    let results = ctx.bob.query().send().await.expect("query");

    assert!(!results.contains(&result.id));

    // Alice can still see her own record.
    let alice_results = ctx.alice.query().send().await.expect("alice query");
    assert!(alice_results.contains(&result.id));
}
