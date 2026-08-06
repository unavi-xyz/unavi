use std::time::Duration;

use rstest::rstest;
use rusqlite::params;
use tracing_test::traced_test;
use wds::{
    DataStore,
    entries,
};

use crate::common::{
    DataStoreCtx,
    ctx,
};

mod common;

async fn hosts(store: &DataStore, ns: &str) -> i64 {
    let ns = ns.to_string();
    store
        .db()
        .call(move |conn| {
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM hosted_docs WHERE ns = ?",
                params![&ns],
                |row| row.get(0),
            )?;
            Ok(count)
        })
        .await
        .expect("count hosted_docs")
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn unhost_by_a_non_host_is_denied(#[future] ctx: DataStoreCtx) {
    let ns = entries::create(ctx.store.docs()).await.expect("create doc");
    ctx.alice.host_doc(ns).await.expect("alice hosts");

    let result = ctx.bob.unhost_doc(ns).await;

    assert!(result.is_err(), "bob never hosted this doc");
    assert_eq!(
        hosts(&ctx.store, &ns.to_string()).await,
        1,
        "alice's hosting survives bob's attempt"
    );
    assert!(
        ctx.store
            .docs()
            .api()
            .open(ns)
            .await
            .expect("open")
            .is_some(),
        "the namespace was not dropped out from under alice"
    );
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn unhost_drops_only_after_the_last_host_leaves(#[future] ctx: DataStoreCtx) {
    let ns = entries::create(ctx.store.docs()).await.expect("create doc");
    ctx.alice.host_doc(ns).await.expect("alice hosts");
    ctx.bob.host_doc(ns).await.expect("bob hosts");

    ctx.alice.unhost_doc(ns).await.expect("alice unhosts");
    assert!(
        ctx.store
            .docs()
            .api()
            .open(ns)
            .await
            .expect("open")
            .is_some(),
        "bob still hosts it, so it stays"
    );

    ctx.bob.unhost_doc(ns).await.expect("bob unhosts");
    assert_eq!(hosts(&ctx.store, &ns.to_string()).await, 0);
}
