mod common;

use std::time::Duration;

use rstest::rstest;
use rusqlite::{OptionalExtension, params};
use time::OffsetDateTime;
use tracing_test::traced_test;

use crate::common::{DataStoreCtx, ctx};

async fn insert_record_dep(ctx: &DataStoreCtx, record_id: &str, dep_record_id: &str) {
    let record_id = record_id.to_string();
    let dep_record_id = dep_record_id.to_string();
    ctx.store
        .db()
        .call(move |conn| {
            conn.execute(
                "INSERT OR IGNORE INTO record_record_deps
                 (record_id, dep_record_id) VALUES (?, ?)",
                params![&record_id, &dep_record_id],
            )?;
            Ok(())
        })
        .await
        .expect("insert record dep");
}

async fn count_record_deps(ctx: &DataStoreCtx, record_id: &str) -> i64 {
    let record_id = record_id.to_string();
    ctx.store
        .db()
        .call(move |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM record_record_deps
                 WHERE record_id = ?",
                params![&record_id],
                |row| row.get(0),
            )
            .map_err(Into::into)
        })
        .await
        .expect("query")
}

async fn count_as_dep(ctx: &DataStoreCtx, dep_record_id: &str) -> i64 {
    let dep_record_id = dep_record_id.to_string();
    ctx.store
        .db()
        .call(move |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM record_record_deps
                 WHERE dep_record_id = ?",
                params![&dep_record_id],
                |row| row.get(0),
            )
            .map_err(Into::into)
        })
        .await
        .expect("query")
}

async fn record_exists(ctx: &DataStoreCtx, record_id: &str) -> bool {
    let record_id = record_id.to_string();
    let count: i64 = ctx
        .store
        .db()
        .call(move |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM records WHERE id = ?",
                params![&record_id],
                |row| row.get(0),
            )
            .map_err(Into::into)
        })
        .await
        .expect("query");
    count > 0
}

async fn pin_expires(ctx: &DataStoreCtx, owner: &str, record_id: &str) -> Option<i64> {
    let owner = owner.to_string();
    let record_id = record_id.to_string();
    ctx.store
        .db()
        .call(move |conn| {
            conn.query_row(
                "SELECT expires FROM record_pins
                 WHERE owner = ? AND record_id = ?",
                params![&owner, &record_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
        })
        .await
        .expect("query")
}

async fn expire_pin(ctx: &DataStoreCtx, owner: &str, record_id: &str) {
    let past = OffsetDateTime::now_utc().unix_timestamp() - 1;
    let owner = owner.to_string();
    let record_id = record_id.to_string();
    ctx.store
        .db()
        .call(move |conn| {
            conn.execute(
                "UPDATE record_pins SET expires = ?
                 WHERE owner = ? AND record_id = ?",
                params![past, &owner, &record_id],
            )?;
            Ok(())
        })
        .await
        .expect("expire pin");
}

/// Record dep table can be populated and queried.
#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_record_deps_stored(#[future] ctx: DataStoreCtx) {
    let parent = ctx.alice.create_record().send().await.expect("create");
    let dep = ctx.alice.create_record().send().await.expect("create");

    let parent_id = parent.id.to_string();
    let dep_id = dep.id.to_string();

    insert_record_dep(&ctx, &parent_id, &dep_id).await;

    assert_eq!(count_record_deps(&ctx, &parent_id).await, 1);
    assert_eq!(count_as_dep(&ctx, &dep_id).await, 1);
}

/// When parent is still pinned, GC extends the dep record's pin
/// instead of deleting it — same pattern as blob pin extension.
#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_gc_extends_dep_record_pin(#[future] ctx: DataStoreCtx) {
    let parent = ctx.alice.create_record().send().await.expect("create");
    let dep = ctx.alice.create_record().send().await.expect("create");

    let parent_id = parent.id.to_string();
    let dep_id = dep.id.to_string();
    let alice_did = ctx.alice.identity().did().to_string();

    insert_record_dep(&ctx, &parent_id, &dep_id).await;

    // Parent pin is far in the future.
    let parent_expires = pin_expires(&ctx, &alice_did, &parent_id)
        .await
        .expect("parent pinned");

    // Expire the dep record's pin.
    expire_pin(&ctx, &alice_did, &dep_id).await;

    // Run GC.
    ctx.store.run_gc().await.expect("gc");

    // Dep record should still exist.
    assert!(record_exists(&ctx, &dep_id).await, "dep should survive");

    // Dep pin should be extended to match parent's expiration.
    let new_expires = pin_expires(&ctx, &alice_did, &dep_id)
        .await
        .expect("dep pin should still exist");
    assert_eq!(
        new_expires, parent_expires,
        "dep pin should match parent expiration"
    );
}

/// When both parent and dep pins expire, the dep record is deleted.
#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_gc_deletes_dep_when_parent_expired(#[future] ctx: DataStoreCtx) {
    let parent = ctx.alice.create_record().send().await.expect("create");
    let dep = ctx.alice.create_record().send().await.expect("create");

    let parent_id = parent.id.to_string();
    let dep_id = dep.id.to_string();
    let alice_did = ctx.alice.identity().did().to_string();

    insert_record_dep(&ctx, &parent_id, &dep_id).await;

    // Expire both pins.
    expire_pin(&ctx, &alice_did, &parent_id).await;
    expire_pin(&ctx, &alice_did, &dep_id).await;

    // Run GC.
    ctx.store.run_gc().await.expect("gc");

    // Both should be deleted.
    assert!(
        !record_exists(&ctx, &parent_id).await,
        "parent should be deleted"
    );
    assert!(!record_exists(&ctx, &dep_id).await, "dep should be deleted");

    // record_record_deps rows should be cleaned up.
    assert_eq!(
        count_record_deps(&ctx, &parent_id).await,
        0,
        "dep rows should be cleaned up"
    );
}

/// An independent pin from another owner keeps the dep record alive
/// even after the parent's owner's pin expires.
#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_independent_pin_keeps_dep_alive(#[future] ctx: DataStoreCtx) {
    let parent = ctx.alice.create_record().send().await.expect("create");
    let dep = ctx.alice.create_record().send().await.expect("create");

    let parent_id = parent.id.to_string();
    let dep_id = dep.id.to_string();
    let alice_did = ctx.alice.identity().did().to_string();
    let bob_did = ctx.bob.identity().did().to_string();

    insert_record_dep(&ctx, &parent_id, &dep_id).await;

    // Bob independently pins the dep record.
    let far_future = OffsetDateTime::now_utc().unix_timestamp() + 86400;
    ctx.store
        .db()
        .call({
            let bob_did = bob_did.clone();
            let dep_id = dep_id.clone();
            move |conn| {
                conn.execute(
                    "INSERT INTO record_pins (record_id, owner, expires)
                     VALUES (?, ?, ?)",
                    params![&dep_id, &bob_did, far_future],
                )?;
                Ok(())
            }
        })
        .await
        .expect("bob pin");

    // Expire Alice's pins on both parent and dep.
    expire_pin(&ctx, &alice_did, &parent_id).await;
    expire_pin(&ctx, &alice_did, &dep_id).await;

    // Run GC.
    ctx.store.run_gc().await.expect("gc");

    // Parent should be deleted (Alice was the only pinner).
    assert!(
        !record_exists(&ctx, &parent_id).await,
        "parent should be deleted"
    );

    // Dep should survive because Bob still pins it.
    assert!(
        record_exists(&ctx, &dep_id).await,
        "dep should survive via Bob's independent pin"
    );

    // Bob's pin should be untouched.
    let bob_expires = pin_expires(&ctx, &bob_did, &dep_id)
        .await
        .expect("bob pin should exist");
    assert_eq!(bob_expires, far_future);
}

/// Cross-owner: Alice pins parent which depends on dep. Bob doesn't
/// pin the parent but does have the dep pinned. When Alice's dep pin
/// expires, GC extends it because Alice's parent still needs it.
/// Bob's pin is unrelated to this — the extension comes from Alice's
/// own parent record.
#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_cross_owner_dep_extension(#[future] ctx: DataStoreCtx) {
    let parent = ctx.alice.create_record().send().await.expect("create");
    let dep = ctx.alice.create_record().send().await.expect("create");

    let parent_id = parent.id.to_string();
    let dep_id = dep.id.to_string();
    let alice_did = ctx.alice.identity().did().to_string();
    let bob_did = ctx.bob.identity().did().to_string();

    insert_record_dep(&ctx, &parent_id, &dep_id).await;

    let alice_parent_expires = pin_expires(&ctx, &alice_did, &parent_id)
        .await
        .expect("alice parent pinned");

    // Bob pins dep with a shorter expiration.
    let bob_exp = OffsetDateTime::now_utc().unix_timestamp() + 60;
    ctx.store
        .db()
        .call({
            let bob_did = bob_did.clone();
            let dep_id = dep_id.clone();
            move |conn| {
                conn.execute(
                    "INSERT INTO record_pins (record_id, owner, expires)
                     VALUES (?, ?, ?)",
                    params![&dep_id, &bob_did, bob_exp],
                )?;
                Ok(())
            }
        })
        .await
        .expect("bob pin");

    // Expire Alice's dep pin.
    expire_pin(&ctx, &alice_did, &dep_id).await;

    // Run GC.
    ctx.store.run_gc().await.expect("gc");

    // Alice's dep pin should be extended to match her parent.
    let alice_dep_expires = pin_expires(&ctx, &alice_did, &dep_id)
        .await
        .expect("alice dep pin should be extended");
    assert_eq!(
        alice_dep_expires, alice_parent_expires,
        "alice dep pin should match parent expiration"
    );

    // Bob's pin should be untouched.
    let bob_expires = pin_expires(&ctx, &bob_did, &dep_id)
        .await
        .expect("bob pin should exist");
    assert_eq!(bob_expires, bob_exp, "bob's pin should be untouched");
}

/// GC cleans up record_record_deps rows (both directions) when a
/// record is fully deleted.
#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_gc_cleans_dep_rows(#[future] ctx: DataStoreCtx) {
    let a = ctx.alice.create_record().send().await.expect("create");
    let b = ctx.alice.create_record().send().await.expect("create");
    let c = ctx.alice.create_record().send().await.expect("create");

    let a_id = a.id.to_string();
    let b_id = b.id.to_string();
    let c_id = c.id.to_string();
    let alice_did = ctx.alice.identity().did().to_string();

    // A depends on B, C depends on B.
    insert_record_dep(&ctx, &a_id, &b_id).await;
    insert_record_dep(&ctx, &c_id, &b_id).await;

    assert_eq!(count_as_dep(&ctx, &b_id).await, 2);

    // Expire and GC record A only.
    expire_pin(&ctx, &alice_did, &a_id).await;
    // Also expire B so it doesn't get extended by A's (expired) parent.
    expire_pin(&ctx, &alice_did, &b_id).await;

    ctx.store.run_gc().await.expect("gc");

    // A should be deleted.
    assert!(!record_exists(&ctx, &a_id).await, "A should be deleted");
    // A's dep row (A→B) should be cleaned up.
    assert_eq!(count_record_deps(&ctx, &a_id).await, 0);
    // But C→B should still exist (C is still pinned).
    assert_eq!(count_as_dep(&ctx, &b_id).await, 1);
    // B should survive because C still pins it (C is not expired,
    // and C depends on B so B's pin got extended).
    assert!(record_exists(&ctx, &b_id).await, "B should survive via C");
}
