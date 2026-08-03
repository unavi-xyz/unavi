use std::time::Duration;

use bytes::Bytes;
use rstest::rstest;
use tracing_test::traced_test;
use wds::space;

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
async fn test_snapshot_doc_roundtrip(#[future] ctx: DataStoreCtx) {
    let snapshot = Bytes::from_static(b"loro-snapshot-bytes");

    let ns = space::create_snapshot_doc(
        ctx.store.docs(),
        ctx.store.blobs().blobs(),
        snapshot.clone(),
        &[],
    )
    .await
    .expect("create snapshot doc");

    let read = space::read_snapshot(ctx.store.docs(), ctx.store.blobs().blobs(), ns)
        .await
        .expect("read snapshot")
        .expect("snapshot present");

    assert_eq!(read, snapshot);
}
