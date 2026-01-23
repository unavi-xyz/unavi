mod common;

use std::time::Duration;

use rstest::rstest;
use rusqlite::params;
use tracing_test::traced_test;

use crate::common::{DataStoreCtx, ctx};

#[rstest]
#[timeout(Duration::from_secs(10))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_schema_deps_tracked_with_type(#[future] ctx: DataStoreCtx) {
    let result = ctx.alice.create_record().send().await.expect("create");
    let id_str = result.id.to_string();

    let schema_dep_count: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM record_blob_deps WHERE record_id = ? AND dep_type = 'schema'",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");

    assert!(
        schema_dep_count >= 2,
        "expected at least 2 schema deps, got {schema_dep_count}"
    );
}

#[rstest]
#[timeout(Duration::from_secs(10))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_ref_deps_sql_operations(#[future] ctx: DataStoreCtx) {
    let result = ctx.alice.create_record().send().await.expect("create");
    let id_str = result.id.to_string();

    let fake_hash = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
    ctx.store
        .db()
        .call({
            let id_str = id_str.clone();
            let fake_hash = fake_hash.to_string();
            move |conn| {
                conn.execute(
                    "INSERT OR IGNORE INTO record_blob_deps (record_id, blob_hash, dep_type)
                     VALUES (?, ?, 'ref')",
                    params![&id_str, &fake_hash],
                )?;
                Ok(())
            }
        })
        .await
        .expect("insert ref dep");

    let ref_dep_count: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM record_blob_deps WHERE record_id = ? AND dep_type = 'ref'",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");

    assert_eq!(ref_dep_count, 1, "expected 1 ref dep");

    let total_dep_count: i64 = ctx
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

    assert!(
        total_dep_count >= 3,
        "expected at least 3 total deps (2 schema + 1 ref), got {total_dep_count}"
    );
}

#[rstest]
#[timeout(Duration::from_secs(10))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_ref_deps_delete_only_refs(#[future] ctx: DataStoreCtx) {
    let result = ctx.alice.create_record().send().await.expect("create");
    let id_str = result.id.to_string();

    for i in 0..3 {
        let fake_hash =
            format!("af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f326{i}");
        ctx.store
            .db()
            .call({
                let id_str = id_str.clone();
                move |conn| {
                    conn.execute(
                        "INSERT OR IGNORE INTO record_blob_deps (record_id, blob_hash, dep_type)
                         VALUES (?, ?, 'ref')",
                        params![&id_str, &fake_hash],
                    )?;
                    Ok(())
                }
            })
            .await
            .expect("insert ref dep");
    }

    let schema_count_before: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM record_blob_deps WHERE record_id = ? AND dep_type = 'schema'",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");

    ctx.store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.execute(
                    "DELETE FROM record_blob_deps WHERE record_id = ? AND dep_type = 'ref'",
                    params![&id_str],
                )?;
                Ok(())
            }
        })
        .await
        .expect("delete ref deps");

    let ref_count_after: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM record_blob_deps WHERE record_id = ? AND dep_type = 'ref'",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");

    assert_eq!(ref_count_after, 0, "ref deps should be deleted");

    let schema_count_after: i64 = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM record_blob_deps WHERE record_id = ? AND dep_type = 'schema'",
                    params![&id_str],
                    |row| row.get(0),
                )
                .map_err(Into::into)
            }
        })
        .await
        .expect("query");

    assert_eq!(
        schema_count_before, schema_count_after,
        "schema deps should not be deleted"
    );
}

#[rstest]
#[timeout(Duration::from_secs(10))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_blob_dep_query_returns_all_types(#[future] ctx: DataStoreCtx) {
    let result = ctx.alice.create_record().send().await.expect("create");
    let id_str = result.id.to_string();

    let ref_hash = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262";
    ctx.store
        .db()
        .call({
            let id_str = id_str.clone();
            let ref_hash = ref_hash.to_string();
            move |conn| {
                conn.execute(
                    "INSERT OR IGNORE INTO record_blob_deps (record_id, blob_hash, dep_type)
                     VALUES (?, ?, 'ref')",
                    params![&id_str, &ref_hash],
                )?;
                Ok(())
            }
        })
        .await
        .expect("insert ref dep");

    let all_hashes: Vec<String> = ctx
        .store
        .db()
        .call({
            let id_str = id_str.clone();
            move |conn| {
                let mut stmt =
                    conn.prepare("SELECT blob_hash FROM record_blob_deps WHERE record_id = ?")?;
                let rows = stmt.query_map(params![&id_str], |row| row.get(0))?;
                rows.collect::<Result<Vec<String>, _>>().map_err(Into::into)
            }
        })
        .await
        .expect("query");

    assert!(
        all_hashes.contains(&ref_hash.to_string()),
        "all_hashes should include ref dep"
    );
    assert!(
        all_hashes.len() >= 3,
        "should have at least 2 schema + 1 ref"
    );
}
