// use std::time::Duration;
//
// use loro::{LoroDoc, LoroList};
// use rstest::rstest;
// use tracing_test::traced_test;
// use wds::{
//     record::{acl::Acl, envelope::Envelope},
//     signed_bytes::Signable,
//     sync::shared::store_envelope,
// };
//
// use crate::common::{DataStoreCtx, acl_only, acl_with_reader, acl_with_writer, ctx};
//
// mod common;
//
// /// Unauthorized user cannot write to a record.
// #[rstest]
// #[timeout(Duration::from_secs(5))]
// #[awt]
// #[traced_test]
// #[tokio::test]
// async fn test_write_acl_denies_unauthorized(#[future] ctx: DataStoreCtx) {
//     // Alice creates record with only herself in ACL.
//     let acl = acl_only(&ctx.alice.did);
//     let (record_id, doc) = ctx
//         .alice
//         .create_record(&ctx.store, acl, vec![])
//         .await
//         .expect("create record");
//
//     // Bob modifies the document and tries to submit.
//     doc.get_map("test")
//         .insert("key", "value")
//         .expect("insert into map");
//     let envelope = Envelope::all_updates(ctx.bob.did.clone(), &doc).expect("build envelope");
//     let signed = envelope.sign(&ctx.bob.signing_key).expect("sign envelope");
//     let env_bytes = postcard::to_stdvec(&signed).expect("serialize envelope");
//
//     let result = store_envelope(
//         ctx.store.db(),
//         ctx.store.blobs(),
//         &record_id.to_string(),
//         &env_bytes,
//     )
//     .await;
//
//     assert!(result.is_err(), "unauthorized write should fail");
//     assert!(
//         result
//             .expect_err("invariant")
//             .to_string()
//             .contains("access denied"),
//         "error should mention access denied"
//     );
// }
//
// /// Authorized user in write ACL can write to a record.
// #[rstest]
// #[timeout(Duration::from_secs(5))]
// #[awt]
// #[traced_test]
// #[tokio::test]
// async fn test_write_acl_allows_authorized(#[future] ctx: DataStoreCtx) {
//     // Alice creates record with Bob in write ACL.
//     let acl = acl_with_writer(&ctx.alice.did, &ctx.bob.did);
//     let (record_id, doc) = ctx
//         .alice
//         .create_record(&ctx.store, acl, vec![])
//         .await
//         .expect("create record");
//
//     // Bob modifies the document and submits.
//     doc.get_map("test")
//         .insert("key", "value")
//         .expect("insert into map");
//     let envelope = Envelope::all_updates(ctx.bob.did.clone(), &doc).expect("build envelope");
//     let signed = envelope.sign(&ctx.bob.signing_key).expect("sign envelope");
//
//     let result = ctx
//         .bob
//         .submit_envelope(&ctx.store, &record_id, &signed)
//         .await;
//
//     assert!(result.is_ok(), "authorized write should succeed");
// }
//
// /// User with only read permission cannot write.
// #[rstest]
// #[timeout(Duration::from_secs(5))]
// #[awt]
// #[traced_test]
// #[tokio::test]
// async fn test_read_only_cannot_write(#[future] ctx: DataStoreCtx) {
//     // Alice creates record with Bob only in read ACL.
//     let acl = acl_with_reader(&ctx.alice.did, &ctx.bob.did);
//     let (record_id, doc) = ctx
//         .alice
//         .create_record(&ctx.store, acl, vec![])
//         .await
//         .expect("create record");
//
//     // Bob tries to write.
//     doc.get_map("test")
//         .insert("key", "value")
//         .expect("insert into map");
//     let envelope = Envelope::all_updates(ctx.bob.did.clone(), &doc).expect("build envelope");
//     let signed = envelope.sign(&ctx.bob.signing_key).expect("sign envelope");
//     let env_bytes = postcard::to_stdvec(&signed).expect("serialize envelope");
//
//     let result = store_envelope(
//         ctx.store.db(),
//         ctx.store.blobs(),
//         &record_id.to_string(),
//         &env_bytes,
//     )
//     .await;
//
//     assert!(result.is_err(), "read-only user write should fail");
// }
//
// /// ACL modification requires manage permission.
// #[rstest]
// #[timeout(Duration::from_secs(5))]
// #[awt]
// #[traced_test]
// #[tokio::test]
// async fn test_acl_modification_requires_manage(#[future] ctx: DataStoreCtx) {
//     // Alice creates record with Bob in write but not manage.
//     let acl = acl_with_writer(&ctx.alice.did, &ctx.bob.did);
//     let (record_id, doc) = ctx
//         .alice
//         .create_record(&ctx.store, acl, vec![])
//         .await
//         .expect("create record");
//
//     // Bob tries to modify ACL to add himself to manage.
//     let acl_map = doc.get_map("acl");
//     let manage_list: LoroList = acl_map
//         .get_or_create_container("manage", LoroList::new())
//         .expect("get container");
//     manage_list
//         .push(ctx.bob.did.to_string())
//         .expect("push to list");
//
//     let envelope = Envelope::all_updates(ctx.bob.did.clone(), &doc).expect("build envelope");
//     let signed = envelope.sign(&ctx.bob.signing_key).expect("sign envelope");
//     let env_bytes = postcard::to_stdvec(&signed).expect("serialize envelope");
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
//         "ACL modification without manage permission should fail"
//     );
// }
//
// /// Privilege escalation in a single envelope is prevented.
// /// ACL is checked against OLD state before envelope is applied.
// #[rstest]
// #[timeout(Duration::from_secs(5))]
// #[awt]
// #[traced_test]
// #[tokio::test]
// async fn test_privilege_escalation_prevented(#[future] ctx: DataStoreCtx) {
//     // Alice creates record with Bob in write ACL.
//     let acl = acl_with_writer(&ctx.alice.did, &ctx.bob.did);
//     let (record_id, doc) = ctx
//         .alice
//         .create_record(&ctx.store, acl, vec![])
//         .await
//         .expect("create record");
//
//     // Bob creates an envelope that:
//     // 1. Adds himself to manage ACL.
//     // 2. Makes other changes that would require manage.
//     // This should fail because ACL is checked against OLD state.
//     let acl_map = doc.get_map("acl");
//     let manage_list: LoroList = acl_map
//         .get_or_create_container("manage", LoroList::new())
//         .expect("get container");
//     manage_list
//         .push(ctx.bob.did.to_string())
//         .expect("push to list");
//
//     let envelope = Envelope::all_updates(ctx.bob.did.clone(), &doc).expect("build envelope");
//     let signed = envelope.sign(&ctx.bob.signing_key).expect("sign envelope");
//     let env_bytes = postcard::to_stdvec(&signed).expect("serialize envelope");
//
//     let result = store_envelope(
//         ctx.store.db(),
//         ctx.store.blobs(),
//         &record_id.to_string(),
//         &env_bytes,
//     )
//     .await;
//
//     assert!(result.is_err(), "privilege escalation should be prevented");
// }
//
// /// Manager can modify ACL.
// #[rstest]
// #[timeout(Duration::from_secs(5))]
// #[awt]
// #[traced_test]
// #[tokio::test]
// async fn test_manager_can_modify_acl(#[future] ctx: DataStoreCtx) {
//     // Alice creates record where she is manager.
//     let acl = acl_only(&ctx.alice.did);
//     let (record_id, doc) = ctx
//         .alice
//         .create_record(&ctx.store, acl, vec![])
//         .await
//         .expect("create record");
//
//     // Alice adds Bob to write ACL.
//     let acl_map = doc.get_map("acl");
//     let write_list: LoroList = acl_map
//         .get_or_create_container("write", LoroList::new())
//         .expect("get container");
//     write_list
//         .push(ctx.bob.did.to_string())
//         .expect("push to list");
//
//     let envelope = Envelope::all_updates(ctx.alice.did.clone(), &doc).expect("build envelope");
//     let signed = envelope
//         .sign(&ctx.alice.signing_key)
//         .expect("sign envelope");
//
//     let result = ctx
//         .alice
//         .submit_envelope(&ctx.store, &record_id, &signed)
//         .await;
//
//     assert!(result.is_ok(), "manager should be able to modify ACL");
// }
//
// /// Record not pinned cannot receive envelopes.
// #[rstest]
// #[timeout(Duration::from_secs(5))]
// #[awt]
// #[traced_test]
// #[tokio::test]
// async fn test_unpinned_record_rejected(#[future] ctx: DataStoreCtx) {
//     // Create an envelope for a non-existent, unpinned record.
//     let doc = LoroDoc::new();
//     let record = wds::record::Record::new(ctx.alice.did.clone());
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
//     let record_id = record.id().expect("get record id");
//
//     // Try to store without pinning first.
//     let result = store_envelope(
//         ctx.store.db(),
//         ctx.store.blobs(),
//         &record_id.to_string(),
//         &env_bytes,
//     )
//     .await;
//
//     assert!(result.is_err(), "unpinned record should reject envelopes");
//     assert!(
//         result
//             .expect_err("invariant")
//             .to_string()
//             .contains("not pinned"),
//         "error should mention record not pinned"
//     );
// }
