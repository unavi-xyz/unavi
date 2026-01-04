use std::time::Duration;

use rstest::rstest;
use tracing_test::traced_test;
use wds::record::{acl::Acl, schema::{SCHEMA_BEACON, SCHEMA_HOME}};

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
    let results = ctx.alice.query_records(None, &[]).await.expect("query");

    assert!(results.contains(&record_id));
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_query_by_creator(#[future] ctx: DataStoreCtx) {
    // Alice and Bob each create a record.
    let alice_result = ctx.alice.create_record().send().await.expect("alice create");
    let bob_result = ctx.bob.create_record().send().await.expect("bob create");

    // Alice queries for her own records.
    let results = ctx
        .alice
        .query_records(Some(ctx.alice.did()), &[])
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
        .add_schema(&*SCHEMA_HOME, |_| Ok(()))
        .expect("add schema")
        .send()
        .await
        .expect("create home");

    // Alice creates a record with SCHEMA_BEACON.
    let beacon_result = ctx
        .alice
        .create_record()
        .add_schema(&*SCHEMA_BEACON, |_| Ok(()))
        .expect("add schema")
        .send()
        .await
        .expect("create beacon");

    // Query for HOME schema.
    let results = ctx
        .alice
        .query_records(None, &[SCHEMA_HOME.hash])
        .await
        .expect("query");

    assert!(results.contains(&home_result.id));
    assert!(!results.contains(&beacon_result.id));
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
    let results = ctx.bob.query_records(None, &[]).await.expect("query");

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
    acl.read.push(ctx.bob.did().clone());
    acl.save(&doc).expect("save acl");

    ctx.alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("update acl");

    // Bob queries - should now see Alice's record.
    let results = ctx.bob.query_records(None, &[]).await.expect("query");

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
        .query_records(None, &[fake_hash])
        .await
        .expect("query");

    assert!(results.is_empty());
}
