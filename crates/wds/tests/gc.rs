use std::time::Duration;

use rand::RngCore;
use rstest::rstest;
use rusqlite::params;
use time::OffsetDateTime;
use tracing_test::traced_test;
use wds::tag::BlobTag;

use crate::common::{
    DataStoreCtx,
    ctx,
};

mod common;

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_gc_removes_expired_blob_pin(#[future] ctx: DataStoreCtx) {
    let did = ctx.alice.identity().did().clone();
    let did_str = did.to_string();

    let mut bytes = vec![0u8; 1024];
    rand::rng().fill_bytes(&mut bytes);
    let hash = ctx
        .alice
        .upload_blob(bytes.into())
        .await
        .expect("upload blob");

    let tag = BlobTag::new(did, hash).to_string();
    assert!(
        ctx.store
            .blobs()
            .tags()
            .get(tag.clone())
            .await
            .expect("get tag")
            .is_some(),
        "tag should exist after upload"
    );

    let past = OffsetDateTime::now_utc().unix_timestamp() - 1;
    ctx.store
        .db()
        .call({
            let did_str = did_str.clone();
            move |conn| {
                conn.execute(
                    "UPDATE blob_pins SET expires = ? WHERE owner = ?",
                    params![past, &did_str],
                )?;
                Ok(())
            }
        })
        .await
        .expect("expire pin");

    ctx.store.run_gc().await.expect("run gc");

    let remaining: i64 = ctx
        .store
        .db()
        .call(move |conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM blob_pins WHERE owner = ?",
                params![&did_str],
                |row| row.get(0),
            )
            .map_err(Into::into)
        })
        .await
        .expect("count pins");
    assert_eq!(remaining, 0, "expired pin should be removed");

    assert!(
        ctx.store
            .blobs()
            .tags()
            .get(tag)
            .await
            .expect("get tag")
            .is_none(),
        "tag should be removed after gc"
    );
}
