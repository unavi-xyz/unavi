// use std::time::Duration;
//
// use loro::LoroDoc;
// use rstest::rstest;
// use tracing_test::traced_test;
// use wds::{
//     record::{Record, acl::Acl, envelope::Envelope},
//     signed_bytes::{Signable, SignedBytes},
//     sync::shared::store_envelope,
// };
//
// use crate::common::{DataStoreCtx, acl_only, ctx};
//
// mod common;
//
// /// Envelope with tampered signature is rejected.
// #[rstest]
// #[timeout(Duration::from_secs(5))]
// #[awt]
// #[traced_test]
// #[tokio::test]
// async fn test_invalid_signature_rejected(#[future] ctx: DataStoreCtx) {
//     // Alice creates record.
//     let acl = acl_only(&ctx.alice.did);
//     let (record_id, doc) = ctx
//         .alice
//         .create_record(&ctx.store, acl, Vec::new())
//         .await
//         .expect("create record");
//
//     // Create a valid envelope then tamper with it.
//     doc.get_map("test")
//         .insert("key", "value")
//         .expect("insert into map");
//
//     let envelope = Envelope::all_updates(ctx.alice.did.clone(), &doc).expect("build envelope");
//     let signed = envelope
//         .sign(&ctx.alice.signing_key)
//         .expect("sign envelope");
//
//     // Tamper with the signature.
//     let mut sig = signed.signature().to_vec();
//     if let Some(byte) = sig.first_mut() {
//         *byte ^= 0xFF;
//     }
//
//     let tampered = SignedBytes::<Envelope>::from_parts(signed.payload_bytes().to_vec(), sig);
//
//     let env_bytes = postcard::to_stdvec(&tampered).expect("serialize envelope");
//
//     let result = store_envelope(
//         ctx.store.db(),
//         ctx.store.blobs(),
//         &record_id.to_string(),
//         &env_bytes,
//     )
//     .await;
//
//     assert!(result.is_err(), "tampered signature should be rejected");
//     assert!(
//         result
//             .expect_err("invariant")
//             .to_string()
//             .contains("signature"),
//         "error should mention signature"
//     );
// }
//
// /// Envelope signed by wrong author is rejected.
// /// (Envelope claims Bob as author but is signed by Alice.)
// #[rstest]
// #[timeout(Duration::from_secs(5))]
// #[awt]
// #[traced_test]
// #[tokio::test]
// async fn test_wrong_author_signature_rejected(#[future] ctx: DataStoreCtx) {
//     // Alice creates record with Bob as authorized writer.
//     let mut acl = Acl::default();
//     acl.manage.push(ctx.alice.did.clone());
//     acl.write.push(ctx.alice.did.clone());
//     acl.write.push(ctx.bob.did.clone());
//
//     let (record_id, doc) = ctx
//         .alice
//         .create_record(&ctx.store, acl, vec![])
//         .await
//         .expect("create record");
//
//     // Alice creates an envelope claiming to be from Bob.
//     doc.get_map("test")
//         .insert("key", "value")
//         .expect("insert into map");
//
//     let envelope = Envelope::all_updates(ctx.bob.did.clone(), &doc).expect("build envelope");
//     let misattributed = envelope
//         .sign(&ctx.alice.signing_key)
//         .expect("sign envelope");
//
//     let env_bytes = postcard::to_stdvec(&misattributed).expect("serialize envelope");
//
//     let result = store_envelope(
//         ctx.store.db(),
//         ctx.store.blobs(),
//         &record_id.to_string(),
//         &env_bytes,
//     )
//     .await;
//
//     assert!(
//         result.is_err(),
//         "misattributed signature should be rejected"
//     );
//     assert!(
//         result
//             .expect_err("invariant")
//             .to_string()
//             .contains("signature"),
//         "error should mention signature"
//     );
// }
//
// /// Record ID must match computed ID from record metadata.
// #[rstest]
// #[timeout(Duration::from_secs(5))]
// #[awt]
// #[traced_test]
// #[tokio::test]
// async fn test_record_id_mismatch_rejected(#[future] ctx: DataStoreCtx) {
//     // Create a record document.
//     let doc = LoroDoc::new();
//     let record = Record::new(ctx.alice.did.clone());
//     record.save(&doc).expect("save record");
//
//     let acl = acl_only(&ctx.alice.did);
//     acl.save(&doc).expect("save acl");
//
//     let envelope = Envelope::all_updates(ctx.alice.did.clone(), &doc).expect("build envelope");
//     let signed = envelope
//         .sign(&ctx.alice.signing_key)
//         .expect("sign envelope");
//     let env_bytes = postcard::to_stdvec(&signed).expect("serialize envelope");
//
//     // Pin with a DIFFERENT (fake) record ID.
//     let fake_id = blake3::hash(b"fake record id");
//     let fake_id_str = fake_id.to_string();
//     let did_str = ctx.alice.did.to_string();
//     let expires = (time::OffsetDateTime::now_utc() + time::Duration::hours(1)).unix_timestamp();
//
//     sqlx::query!(
//         "INSERT INTO record_pins (record_id, owner, expires) VALUES (?, ?, ?)",
//         fake_id_str,
//         did_str,
//         expires
//     )
//     .execute(ctx.store.db())
//     .await
//     .expect("insert pin");
//
//     // Try to store with mismatched ID.
//     let result = store_envelope(ctx.store.db(), ctx.store.blobs(), &fake_id_str, &env_bytes).await;
//
//     assert!(result.is_err(), "mismatched record ID should be rejected");
//     assert!(
//         result
//             .expect_err("invariant")
//             .to_string()
//             .contains("record ID does not match"),
//         "error should mention ID mismatch"
//     );
// }
//
// /// Quota exceeded prevents envelope storage.
// #[rstest]
// #[timeout(Duration::from_secs(5))]
// #[awt]
// #[traced_test]
// #[tokio::test]
// async fn test_quota_exceeded_rejected(#[future] ctx: DataStoreCtx) {
//     let did_str = ctx.alice.did.to_string();
//
//     // Set a very low quota for Alice.
//     sqlx::query!(
//         "INSERT INTO user_quotas (owner, bytes_used, quota_bytes) VALUES (?, 0, 100)",
//         did_str
//     )
//     .execute(ctx.store.db())
//     .await
//     .expect("insert quota");
//
//     // Create a record document that exceeds quota.
//     let doc = LoroDoc::new();
//     let record = Record::new(ctx.alice.did.clone());
//     record.save(&doc).expect("save record");
//
//     let acl = acl_only(&ctx.alice.did);
//     acl.save(&doc).expect("save acl");
//
//     // Add some data to make it larger.
//     let data_map = doc.get_map("data");
//     data_map
//         .insert("large_field", "x".repeat(1000))
//         .expect("insert into map");
//
//     let envelope = Envelope::all_updates(ctx.alice.did.clone(), &doc).expect("build envelope");
//     let signed = envelope
//         .sign(&ctx.alice.signing_key)
//         .expect("sign envelope");
//     let env_bytes = postcard::to_stdvec(&signed).expect("serialize envelope");
//
//     let record_id = record.id().expect("get record id");
//     let record_id_str = record_id.to_string();
//     let expires = (time::OffsetDateTime::now_utc() + time::Duration::hours(1)).unix_timestamp();
//
//     // Pin the record.
//     sqlx::query!(
//         "INSERT INTO record_pins (record_id, owner, expires) VALUES (?, ?, ?)",
//         record_id_str,
//         did_str,
//         expires
//     )
//     .execute(ctx.store.db())
//     .await
//     .expect("insert pin");
//
//     // Try to store the envelope.
//     let result = store_envelope(
//         ctx.store.db(),
//         ctx.store.blobs(),
//         &record_id_str,
//         &env_bytes,
//     )
//     .await;
//
//     assert!(result.is_err(), "quota exceeded should reject envelope");
//     assert!(
//         result.expect_err("invariant").to_string().contains("quota"),
//         "error should mention quota"
//     );
// }
//
// /// Multiple pinners share quota responsibility.
// #[rstest]
// #[timeout(Duration::from_secs(5))]
// #[awt]
// #[traced_test]
// #[tokio::test]
// async fn test_multiple_pinners_quota_fallback(#[future] ctx: DataStoreCtx) {
//     let alice_did_str = ctx.alice.did.to_string();
//     let bob_did_str = ctx.bob.did.to_string();
//
//     // Set Alice's quota to be exceeded, Bob's quota has room.
//     sqlx::query!(
//         "INSERT INTO user_quotas (owner, bytes_used, quota_bytes) VALUES (?, 1000, 100)",
//         alice_did_str
//     )
//     .execute(ctx.store.db())
//     .await
//     .expect("insert quota");
//
//     sqlx::query!(
//         "INSERT INTO user_quotas (owner, bytes_used, quota_bytes) VALUES (?, 0, 10000)",
//         bob_did_str
//     )
//     .execute(ctx.store.db())
//     .await
//     .expect("insert quota");
//
//     // Create record with both Alice and Bob having write access.
//     let mut acl = Acl::default();
//     acl.manage.push(ctx.alice.did.clone());
//     acl.write.push(ctx.alice.did.clone());
//     acl.write.push(ctx.bob.did.clone());
//
//     // Create record document.
//     let doc = LoroDoc::new();
//     let record = Record::new(ctx.alice.did.clone());
//     record.save(&doc).expect("save record");
//     acl.save(&doc).expect("save acl");
//
//     let envelope = Envelope::all_updates(ctx.alice.did.clone(), &doc).expect("build envelope");
//     let signed = envelope
//         .sign(&ctx.alice.signing_key)
//         .expect("sign envelope");
//     let env_bytes = postcard::to_stdvec(&signed).expect("serialize envelope");
//
//     let record_id = record.id().expect("get record id");
//     let record_id_str = record_id.to_string();
//     let expires = (time::OffsetDateTime::now_utc() + time::Duration::hours(1)).unix_timestamp();
//
//     // Both Alice and Bob pin the record.
//     sqlx::query!(
//         "INSERT INTO record_pins (record_id, owner, expires) VALUES (?, ?, ?)",
//         record_id_str,
//         alice_did_str,
//         expires
//     )
//     .execute(ctx.store.db())
//     .await
//     .expect("insert pin");
//
//     sqlx::query!(
//         "INSERT INTO record_pins (record_id, owner, expires) VALUES (?, ?, ?)",
//         record_id_str,
//         bob_did_str,
//         expires
//     )
//     .execute(ctx.store.db())
//     .await
//     .expect("insert pin");
//
//     // Envelope should succeed because Bob has quota.
//     let result = store_envelope(
//         ctx.store.db(),
//         ctx.store.blobs(),
//         &record_id_str,
//         &env_bytes,
//     )
//     .await;
//
//     assert!(
//         result.is_ok(),
//         "envelope should succeed when any pinner has quota"
//     );
// }
