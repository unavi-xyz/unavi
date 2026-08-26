use std::str::FromStr;

use tracing_test::traced_test;
use unavi_identity::resolve::{
    resolve,
    resolve_allowing_loopback,
};
use xdid::{
    core::did::Did,
    methods::key::keys::{
        DidKeyPair,
        PublicKey,
        p256::P256KeyPair,
    },
};

use crate::common::did_web::spawn_did_web_server;

mod common;

#[tokio::test]
#[traced_test]
async fn resolves_a_locally_served_did_web() {
    let key = P256KeyPair::generate();
    let server = spawn_did_web_server(&key).await;

    let doc = resolve_allowing_loopback(&server.did)
        .await
        .expect("an operator-named loopback target must resolve");

    assert_eq!(doc.id, server.did);
}

#[tokio::test]
#[traced_test]
async fn strict_resolution_refuses_loopback() {
    let key = P256KeyPair::generate();
    let server = spawn_did_web_server(&key).await;

    assert!(
        resolve(&server.did).await.is_none(),
        "a peer-supplied DID must never reach a loopback target"
    );
}

#[tokio::test]
#[traced_test]
async fn resolves_a_did_key() {
    let did = P256KeyPair::generate().public().to_did();
    let doc = resolve(&did).await.expect("did:key must resolve");
    assert_eq!(doc.id, did);
}

#[tokio::test]
#[traced_test]
async fn unreachable_did_web_resolves_to_none() {
    let did = Did::from_str("did:web:localhost%3A1").expect("valid did");
    assert!(resolve(&did).await.is_none());
}
