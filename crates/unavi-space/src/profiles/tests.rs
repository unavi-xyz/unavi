use std::sync::Arc;

use hsd::{
    format::meta::DocMeta,
    key,
};
use iroh::{
    SecretKey,
    address_lookup::memory::MemoryLookup,
    endpoint::presets::N0DisableRelay,
    protocol::Router,
};
use iroh_docs::Author;
use unavi_identity::{
    identity::Identity,
    profile::{
        ProfileSecret,
        protocol::ProfileProtocol,
    },
};
use unavi_store::StoreBuilder;
use xdid::method::key::{
    DidKeyPair,
    PublicKey,
    p256::P256KeyPair,
};

use super::*;
use crate::test_store::block_on;

async fn store_on(endpoint: Endpoint, secret_key: &SecretKey) -> Store {
    StoreBuilder::new(endpoint, Author::from_bytes(&secret_key.to_bytes()))
        .build()
        .await
        .expect("build store")
}

/// A peer serving `user`'s profile, named `name`.
async fn named_peer(user: &Identity, name: &str) -> Router {
    let secret_key = SecretKey::generate();
    let endpoint = Endpoint::builder(N0DisableRelay)
        .secret_key(secret_key.clone())
        .bind()
        .await
        .expect("bind peer");
    let store = store_on(endpoint.clone(), &secret_key).await;

    let (doc, proof) = ProfileSecret::derive(user)
        .expect("derive")
        .open(&store, user)
        .await
        .expect("open profile");
    let meta = DocMeta {
        name: Some(name.into()),
        ..authorship::recorded_meta(&doc)
            .await
            .expect("read meta")
            .expect("open records the author")
    };
    doc.set(key::META, postcard::to_stdvec(&meta).expect("encode meta"))
        .await
        .expect("write meta");

    store
        .accept(Router::builder(endpoint))
        .accept(
            protocol::ALPN,
            ProfileProtocol::new(&proof).expect("encode proof"),
        )
        .spawn()
}

#[test]
fn a_peers_profile_name_arrives() {
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
    let user = Identity::new(did.clone(), key);

    let (profiles, routers) = block_on(async move {
        let peer = named_peer(&user, "Ada").await;

        let lookup = MemoryLookup::new();
        lookup.add_endpoint_info(peer.endpoint().addr());
        let secret_key = SecretKey::generate();
        let endpoint = Endpoint::builder(N0DisableRelay)
            .secret_key(secret_key.clone())
            .address_lookup(lookup)
            .bind()
            .await
            .expect("bind asker");
        let store = store_on(endpoint.clone(), &secret_key).await;
        // The peer syncs back, and a sync both ends start is settled in favour
        // of one of them by endpoint id, so the asker must answer as the app
        // does.
        let asker = store.accept(Router::builder(endpoint.clone())).spawn();

        let profiles = Profiles::default();
        let ask = Ask {
            endpoint,
            store,
            resolver: Arc::new(Resolver::new().expect("resolver")),
            profiles: profiles.clone(),
        };
        assert!(profiles.begin(user.did(), Duration::ZERO));
        task::spawn(ask.run(user.did().clone(), peer.endpoint().addr(), Duration::ZERO));
        (profiles, (peer, asker))
    });

    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    let name = loop {
        if let Some(name) = profiles.get(&did).and_then(|profile| profile.name) {
            break name;
        }
        assert!(std::time::Instant::now() < deadline, "no profile arrived");
        std::thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(name, "Ada");
    assert!(
        !profiles.begin(&did, Duration::from_secs(600)),
        "a found profile is never asked for again"
    );
    drop(routers);
}

#[test]
fn a_failed_ask_waits_before_asking_again() {
    let profiles = Profiles::default();
    let key = P256KeyPair::generate();
    let did = key.public().to_did();

    assert!(profiles.begin(&did, Duration::ZERO));
    assert!(!profiles.begin(&did, Duration::ZERO), "one ask at a time");
    profiles.set(&did, Lookup::Failed { at: Duration::ZERO });
    assert!(!profiles.begin(&did, RETRY.saturating_sub(Duration::from_secs(1))));
    assert!(profiles.begin(&did, RETRY));
}
