use std::time::Duration;

use rstest::rstest;
use rusqlite::{OptionalExtension, params};
use time::OffsetDateTime;
use tracing_test::traced_test;
use wired_schemas::SCHEMA_HOME;

use crate::common::{DataStoreCtx, ctx};

mod common;

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_gc_expired_record_pin(#[future] ctx: DataStoreCtx) {
    // Alice creates a record with a short TTL.
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let record_id = result.id;
    let id_str = record_id.to_string();

    // Verify record exists.
    let exists: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM records WHERE id = ?",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert_eq!(exists, 1, "record should exist");

    // Set pin to expire immediately.
    let past = OffsetDateTime::now_utc().unix_timestamp() - 1;
    ctx.store
        .db()
        .call(move |conn| {
            conn.execute("UPDATE record_pins SET expires = ?", params![past])?;
            Ok(())
        })
        .await
        .expect("update expires");

    // Run GC.
    ctx.store.run_gc().await.expect("run gc");

    // Verify pin was removed.
    let pin_count: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM record_pins WHERE record_id = ?",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert_eq!(pin_count, 0, "pin should be removed");
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_gc_deletes_unpinned_records(#[future] ctx: DataStoreCtx) {
    // Alice creates a record.
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let record_id = result.id;
    let id_str = record_id.to_string();

    // Verify record and envelope exist.
    let record_exists: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM records WHERE id = ?",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert_eq!(record_exists, 1, "record should exist");

    let envelope_count: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM envelopes WHERE record_id = ?",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert!(envelope_count > 0, "envelopes should exist");

    // Set pin to expire immediately.
    let past = OffsetDateTime::now_utc().unix_timestamp() - 1;
    ctx.store
        .db()
        .call(move |conn| {
            conn.execute("UPDATE record_pins SET expires = ?", params![past])?;
            Ok(())
        })
        .await
        .expect("update expires");

    // Run GC.
    ctx.store.run_gc().await.expect("run gc");

    // Verify record and envelopes were deleted.
    let record_exists: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM records WHERE id = ?",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert_eq!(record_exists, 0, "record should be deleted");

    let envelope_count: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM envelopes WHERE record_id = ?",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert_eq!(envelope_count, 0, "envelopes should be deleted");
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_gc_releases_quota(#[future] ctx: DataStoreCtx) {
    // Get initial quota used.
    let did_str = ctx.alice.identity().did().to_string();
    let initial_used: i64 = ctx
        .store
        .db()
        .call({
            let did_str = did_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT bytes_used FROM user_quotas WHERE owner = ?",
                    params![&did_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");

    // Alice creates a record (uses quota).
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let record_id = result.id;

    // Verify quota increased.
    let after_create: i64 = ctx
        .store
        .db()
        .call({
            let did_str = did_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT bytes_used FROM user_quotas WHERE owner = ?",
                    params![&did_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert!(
        after_create > initial_used,
        "quota should increase after create"
    );

    // Set pin to expire immediately.
    let past = OffsetDateTime::now_utc().unix_timestamp() - 1;
    let id_str = record_id.to_string();
    ctx.store
        .db()
        .call(move |conn| {
            conn.execute(
                "UPDATE record_pins SET expires = ? WHERE record_id = ?",
                params![past, &id_str],
            )?;
            Ok(())
        })
        .await
        .expect("update expires");

    // Run GC.
    ctx.store.run_gc().await.expect("run gc");

    // Verify quota was released.
    let after_gc: i64 = ctx
        .store
        .db()
        .call({
            let did_str = did_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT bytes_used FROM user_quotas WHERE owner = ?",
                    params![&did_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert_eq!(after_gc, initial_used, "quota should be released after gc");
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_gc_blob_pin_extended_by_record(#[future] ctx: DataStoreCtx) {
    // Alice creates a record using SCHEMA_HOME.
    // This uploads the schema blob and creates both record pin and blob pin.
    let result = ctx
        .alice
        .create_record()
        .add_schema("home", &*SCHEMA_HOME, |doc| {
            doc.get_map("home").insert("space", "test")?;
            Ok(())
        })
        .expect("add schema")
        .send()
        .await
        .expect("create record");
    let record_id = result.id;
    let id_str = record_id.to_string();
    let hash_str = SCHEMA_HOME.hash.to_string();

    // Get record pin expiration (far in the future).
    let record_expires: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT expires FROM record_pins WHERE record_id = ?",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");

    // Set blob pin to expire immediately.
    let past = OffsetDateTime::now_utc().unix_timestamp() - 1;
    let did_str = ctx.alice.identity().did().to_string();
    ctx.store
        .db()
        .call({
            let did_str = did_str.clone();
            let hash_str = hash_str.clone();
            move |conn| {
                conn.execute(
                    "UPDATE blob_pins SET expires = ? WHERE owner = ? AND hash = ?",
                    params![past, &did_str, &hash_str],
                )?;
                Ok(())
            }
        })
        .await
        .expect("update expires");

    // Run GC.
    ctx.store.run_gc().await.expect("run gc");

    // Blob pin should be extended, not deleted (record still needs it).
    let blob_pin: Option<i64> = ctx
        .store
        .db()
        .call({
            let did_str = did_str.clone();
            let hash_str = hash_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT expires FROM blob_pins WHERE owner = ? AND hash = ?",
                    params![&did_str, &hash_str],
                    |row| row.get(0),
                )
                .optional()
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");

    assert!(blob_pin.is_some(), "blob pin should still exist");
    let new_expires = blob_pin.expect("always exists");
    assert_eq!(
        new_expires, record_expires,
        "blob pin should be extended to match record pin"
    );
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_gc_blob_pin_deleted_when_orphaned(#[future] ctx: DataStoreCtx) {
    // Upload SCHEMA_HOME to the store.
    ctx.store
        .blobs()
        .blobs()
        .add_slice(&SCHEMA_HOME.bytes)
        .await
        .expect("add schema");

    // Alice creates a record using SCHEMA_HOME.
    let result = ctx
        .alice
        .create_record()
        .add_schema("home", &*SCHEMA_HOME, |doc| {
            doc.get_map("home").insert("space", "test")?;
            Ok(())
        })
        .expect("add schema")
        .send()
        .await
        .expect("create record");
    let record_id = result.id;
    let id_str = record_id.to_string();
    let hash_str = SCHEMA_HOME.hash.to_string();
    let did_str = ctx.alice.identity().did().to_string();

    // Expire both record and blob pins.
    let past = OffsetDateTime::now_utc().unix_timestamp() - 1;
    ctx.store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.execute(
                    "UPDATE record_pins SET expires = ? WHERE record_id = ?",
                    params![past, &id_str],
                )?;
                Ok(())
            }
        })
        .await
        .expect("update expires");
    ctx.store
        .db()
        .call({
            let did_str = did_str.clone();
            let hash_str = hash_str.clone();
            move |conn| {
                conn.execute(
                    "UPDATE blob_pins SET expires = ? WHERE owner = ? AND hash = ?",
                    params![past, &did_str, &hash_str],
                )?;
                Ok(())
            }
        })
        .await
        .expect("update expires");

    // Run GC.
    ctx.store.run_gc().await.expect("run gc");

    // Both record and blob pins should be deleted.
    let record_exists: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM records WHERE id = ?",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert_eq!(record_exists, 0, "record should be deleted");

    let blob_pin_exists: i64 = ctx
        .store
        .db()
        .call({
            let did_str = did_str.clone();
            let hash_str = hash_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM blob_pins WHERE owner = ? AND hash = ?",
                    params![&did_str, &hash_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert_eq!(blob_pin_exists, 0, "blob pin should be deleted");
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_gc_deletes_related_indices(#[future] ctx: DataStoreCtx) {
    // Upload SCHEMA_HOME to the store.
    ctx.store
        .blobs()
        .blobs()
        .add_slice(&SCHEMA_HOME.bytes)
        .await
        .expect("add schema");

    // Alice creates a record using SCHEMA_HOME.
    let result = ctx
        .alice
        .create_record()
        .add_schema("home", &*SCHEMA_HOME, |doc| {
            doc.get_map("home").insert("space", "test")?;
            Ok(())
        })
        .expect("add schema")
        .send()
        .await
        .expect("create record");
    let record_id = result.id;
    let id_str = record_id.to_string();

    // Verify indices exist.
    let schema_count: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM record_schemas WHERE record_id = ?",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert!(schema_count > 0, "record_schemas should exist");

    let acl_count: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM record_acl_read WHERE record_id = ?",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert!(acl_count > 0, "record_acl_read should exist");

    let dep_count: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM record_blob_deps WHERE record_id = ?",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert!(dep_count > 0, "record_blob_deps should exist");

    // Set pin to expire immediately.
    let past = OffsetDateTime::now_utc().unix_timestamp() - 1;
    ctx.store
        .db()
        .call(move |conn| {
            conn.execute("UPDATE record_pins SET expires = ?", params![past])?;
            Ok(())
        })
        .await
        .expect("update expires");

    // Run GC.
    ctx.store.run_gc().await.expect("run gc");

    // Verify all indices were deleted.
    let schema_count: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM record_schemas WHERE record_id = ?",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert_eq!(schema_count, 0, "record_schemas should be deleted");

    let acl_count: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM record_acl_read WHERE record_id = ?",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert_eq!(acl_count, 0, "record_acl_read should be deleted");

    let dep_count: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM record_blob_deps WHERE record_id = ?",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");
    assert_eq!(dep_count, 0, "record_blob_deps should be deleted");
}
