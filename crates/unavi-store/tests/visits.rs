//! The retention sweep reads ages off the visit record, so a document that
//! reads back as never visited is one the sweep can never age out.

use std::time::Duration;

use rstest::rstest;
use tracing_test::traced_test;
use unavi_store::Store;

use crate::common::store;

mod common;

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn a_join_reads_back_as_a_visit(#[future] store: Store) {
    let visited = store.create().await.expect("create").id();
    let ignored = store.create().await.expect("create").id();

    let _joined = store.join(visited, Vec::new()).await.expect("join");

    let visits = store.visits().await.expect("visits");

    let age = visits.get(&visited).expect("the visited namespace");
    assert!(
        *age < Duration::from_secs(60),
        "a join just made must not read as ancient: {age:?}"
    );
    assert!(
        !visits.contains_key(&ignored),
        "a namespace never joined must be absent, not aged"
    );
}
