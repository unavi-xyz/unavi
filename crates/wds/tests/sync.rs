use std::time::Duration;

use rstest::rstest;
use tracing_test::traced_test;
use wds::{
    record::{Record, acl::Acl, envelope::Envelope},
    signed_bytes::{Signable, SignedBytes},
};

use crate::common::{DataStoreCtx, assert_contains, ctx};

mod common;

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_invalid_signature_rejected(#[future] ctx: DataStoreCtx) {
    // Alice creates record.
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let (record_id, doc) = (result.id, result.doc);

    // Create a valid envelope then tamper with it.
    let from_vv = doc.oplog_vv();
    doc.get_map("data")
        .insert("key", "value")
        .expect("insert into map");

    let envelope = Envelope::updates(ctx.alice.identity().did().clone(), &doc, from_vv)
        .expect("build envelope");
    let signed = envelope
        .sign(ctx.alice.identity().signing_key())
        .expect("sign envelope");

    // Tamper with the signature.
    let mut sig = signed.signature().to_vec();
    if let Some(byte) = sig.first_mut() {
        *byte ^= 0xFF;
    }

    let tampered = SignedBytes::<Envelope>::from_parts(signed.payload_bytes().to_vec(), sig);

    let e = ctx
        .alice
        .upload_envelope(record_id, &tampered)
        .await
        .expect_err("should error");

    assert_contains(e, "signature");
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_wrong_author_signature_rejected(#[future] ctx: DataStoreCtx) {
    // Alice creates record with Bob as authorized writer.
    let result = ctx
        .alice
        .create_record()
        .send()
        .await
        .expect("create record");
    let (record_id, doc) = (result.id, result.doc);

    // Add Bob to write ACL.
    let from_vv = doc.oplog_vv();
    let mut acl = Acl::load(&doc).expect("load acl");
    acl.write.push(ctx.bob.identity().did().clone());
    acl.save(&doc).expect("save acl");
    ctx.alice
        .update_record(record_id, &doc, from_vv)
        .await
        .expect("update acl");

    // Alice creates an envelope claiming to be from Bob.
    let from_vv = doc.oplog_vv();
    doc.get_map("data")
        .insert("key", "value")
        .expect("insert into map");

    // Envelope claims Bob as author but is signed by Alice.
    let envelope =
        Envelope::updates(ctx.bob.identity().did().clone(), &doc, from_vv).expect("build envelope");
    let misattributed = envelope
        .sign(ctx.alice.identity().signing_key())
        .expect("sign envelope");

    let e = ctx
        .alice
        .upload_envelope(record_id, &misattributed)
        .await
        .expect_err("should error");

    assert_contains(e, "signature");
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_record_id_mismatch_rejected(#[future] ctx: DataStoreCtx) {
    // Create a record document with a specific nonce.
    let doc = loro::LoroDoc::new();
    let record = Record::new(ctx.alice.identity().did().clone());
    record.save(&doc).expect("save record");

    let mut acl = Acl::default();
    acl.manage.push(ctx.alice.identity().did().clone());
    acl.write.push(ctx.alice.identity().did().clone());
    acl.save(&doc).expect("save acl");

    let envelope =
        Envelope::all_updates(ctx.alice.identity().did().clone(), &doc).expect("build envelope");
    let signed = envelope
        .sign(ctx.alice.identity().signing_key())
        .expect("sign envelope");

    // Pin with a different (fake) record ID.
    let fake_id = blake3::hash(b"fake record id");
    ctx.alice
        .pin_record(fake_id, Duration::from_secs(3600))
        .await
        .expect("pin record");

    // Try to upload with mismatched ID.
    let e = ctx
        .alice
        .upload_envelope(fake_id, &signed)
        .await
        .expect_err("should error");

    assert_contains(e, "record ID");
}

#[rstest]
#[timeout(Duration::from_secs(5))]
#[awt]
#[traced_test]
#[tokio::test]
async fn test_unpinned_record_rejected(#[future] ctx: DataStoreCtx) {
    // Create a record document without pinning.
    let doc = loro::LoroDoc::new();
    let record = Record::new(ctx.alice.identity().did().clone());
    record.save(&doc).expect("save record");

    let mut acl = Acl::default();
    acl.manage.push(ctx.alice.identity().did().clone());
    acl.write.push(ctx.alice.identity().did().clone());
    acl.save(&doc).expect("save acl");

    let envelope =
        Envelope::all_updates(ctx.alice.identity().did().clone(), &doc).expect("build envelope");
    let signed = envelope
        .sign(ctx.alice.identity().signing_key())
        .expect("sign envelope");

    let record_id = record.id().expect("get record id");

    // Try to upload without pinning first.
    let e = ctx
        .alice
        .upload_envelope(record_id, &signed)
        .await
        .expect_err("should error");

    assert_contains(e, "not pinned");
}
