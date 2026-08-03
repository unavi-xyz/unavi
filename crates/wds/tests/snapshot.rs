use std::time::Duration;

use bytes::Bytes;
use rstest::rstest;
use tracing_test::traced_test;
use wds::snapshot;

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

    let ns = snapshot::create_doc(
        ctx.store.docs(),
        ctx.store.blobs().blobs(),
        snapshot.clone(),
        &[],
    )
    .await
    .expect("create snapshot doc");

    let read = snapshot::read(ctx.store.docs(), ctx.store.blobs().blobs(), ns)
        .await
        .expect("read snapshot")
        .expect("snapshot present");

    assert_eq!(read, snapshot);
}
