//! `wired/profile` over two real endpoints, with the profile opened in a real
//! store.

use std::sync::Arc;

use iroh::{
    Endpoint,
    SecretKey,
    address_lookup::memory::MemoryLookup,
    endpoint::presets::N0DisableRelay,
    protocol::Router,
};
use iroh_docs::Author;
use unavi_identity::{
    authorship::AuthorshipError,
    identity::Identity,
    profile::{
        ProfileSecret,
        protocol::{
            self,
            AskError,
            ProfileProtocol,
        },
    },
    resolver::Resolver,
};
use unavi_store::StoreBuilder;
use xdid::method::key::{
    DidKeyPair,
    PublicKey,
    p256::P256KeyPair,
};

fn identity() -> Arc<Identity> {
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
    Arc::new(Identity::new(did, key))
}

/// An endpoint serving `user`'s profile, and the namespace it serves.
async fn serve(user: &Identity) -> (Router, iroh_docs::NamespaceId) {
    let secret_key = SecretKey::generate();
    let author = Author::from_bytes(&secret_key.to_bytes());
    let endpoint = Endpoint::builder(N0DisableRelay)
        .secret_key(secret_key)
        .bind()
        .await
        .expect("bind server");
    let store = StoreBuilder::new(endpoint.clone(), author)
        .build()
        .await
        .expect("build store");

    let secret = ProfileSecret::derive(user).expect("derive");
    let (_doc, proof) = secret.open(&store, user).await.expect("open profile");
    let router = store
        .accept(Router::builder(endpoint))
        .accept(
            protocol::ALPN,
            ProfileProtocol::new(&proof).expect("encode proof"),
        )
        .spawn();
    (router, secret.id())
}

async fn client(server: &Router) -> Endpoint {
    let lookup = MemoryLookup::new();
    lookup.add_endpoint_info(server.endpoint().addr());
    Endpoint::builder(N0DisableRelay)
        .address_lookup(lookup)
        .bind()
        .await
        .expect("bind client")
}

#[tokio::test]
async fn a_peer_answers_with_its_users_profile() {
    let user = identity();
    let (server, ns) = serve(&user).await;
    let client = client(&server).await;

    let found = protocol::ask(
        &client,
        server.endpoint().id(),
        user.did(),
        &Resolver::new().expect("resolver"),
    )
    .await
    .expect("ask");
    assert_eq!(found, ns);
}

#[tokio::test]
async fn a_peer_cannot_answer_for_another_user() {
    let (server, _) = serve(&identity()).await;
    let client = client(&server).await;

    let asked = protocol::ask(
        &client,
        server.endpoint().id(),
        identity().did(),
        &Resolver::new().expect("resolver"),
    )
    .await;
    assert!(
        matches!(asked, Err(AskError::Proof(AuthorshipError::WrongDid))),
        "{asked:?}"
    );
}
