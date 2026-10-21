//! The `wired/auth` handshake over two real endpoints: each end proves its DID
//! and binds the one the other proves.
//!
//! The unit tests in `auth::handshake` only exercise the exchange over
//! in-memory streams, so they cannot catch a protocol handler that closes the
//! connection before the peer reads its verdict.

use std::{
    str::FromStr,
    sync::{
        Arc,
        atomic::{
            AtomicUsize,
            Ordering,
        },
    },
    time::Duration,
};

use iroh::{
    Endpoint,
    address_lookup::memory::MemoryLookup,
    endpoint::{
        Connection,
        presets::N0DisableRelay,
    },
    protocol::{
        AcceptError,
        ProtocolHandler,
        Router,
    },
};
use unavi_identity::{
    auth::{
        self,
        AuthProtocol,
        EndpointAuth,
    },
    identity::Identity,
    resolver::Resolver,
};
use xdid::{
    core::did::Did,
    method::key::{
        DidKeyPair,
        PublicKey,
        p256::P256KeyPair,
    },
};

/// The ALPN the dialer proves itself for. Anything other than `wired/auth`
/// triggers the proof ahead of the dial.
const DIAL_ALPN: &[u8] = b"test/auth/dial";

/// Holds the dialed connection open until the peer closes it, so the dialer's
/// proof has a live connection to run on.
#[derive(Debug, Clone)]
struct Hold;

impl ProtocolHandler for Hold {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        connection.closed().await;
        Ok(())
    }
}

fn identity() -> Arc<Identity> {
    let key = P256KeyPair::generate();
    let did = key.public().to_did();
    Arc::new(Identity::new(did, key))
}

/// An identity the verifier cannot resolve, so it declines the proof.
fn unresolvable_identity() -> Arc<Identity> {
    let key = P256KeyPair::generate();
    let did = Did::from_str("did:web:unresolvable.invalid").expect("did");
    Arc::new(Identity::new(did, key))
}

/// Answers `wired/auth` and counts how often it was asked to.
#[derive(Debug, Clone)]
struct Counted {
    inner: AuthProtocol,
    asked: Arc<AtomicUsize>,
}

impl ProtocolHandler for Counted {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        self.asked.fetch_add(1, Ordering::Relaxed);
        self.inner.accept(connection).await
    }
}

async fn bind(auth: &EndpointAuth, lookup: Option<MemoryLookup>) -> Endpoint {
    let mut builder = Endpoint::builder(N0DisableRelay);
    if let Some(lookup) = lookup {
        builder = builder.address_lookup(lookup);
    }
    auth.install(builder).bind().await.expect("bind endpoint")
}

#[tokio::test]
async fn each_end_binds_the_did_the_other_proves() {
    let resolver = Arc::new(Resolver::new().expect("resolver"));

    let server_identity = identity();
    let server_auth = EndpointAuth::new(Arc::clone(&server_identity), Arc::clone(&resolver));
    let server = bind(&server_auth, None).await;
    let (protocol, _server_task) = server_auth
        .serve(server.clone())
        .expect("serve the server endpoint");
    let _router = Router::builder(server.clone())
        .accept(auth::ALPN, protocol)
        .accept(DIAL_ALPN, Hold)
        .spawn();

    let client_identity = identity();
    let client_auth = EndpointAuth::new(Arc::clone(&client_identity), Arc::clone(&resolver));

    // The handshake dials by endpoint id, so the dialer needs the server's
    // address from somewhere other than the dial.
    let lookup = MemoryLookup::new();
    lookup.add_endpoint_info(server.addr());
    let client = bind(&client_auth, Some(lookup)).await;
    let _client_task = client_auth
        .serve(client.clone())
        .expect("serve the client endpoint");

    client
        .connect(server.addr(), DIAL_ALPN)
        .await
        .expect("dial the server");

    let (server_id, client_id) = (server.id(), client.id());

    assert_eq!(
        client_auth.bindings().did_of(server_id).as_ref(),
        Some(server_identity.did()),
        "the dialer must bind the DID the server proved"
    );
    assert_eq!(
        server_auth.bindings().did_of(client_id).as_ref(),
        Some(client_identity.did()),
        "the server must bind the DID the dialer proved"
    );
}

/// Serves `identity`'s `wired/auth` on a fresh endpoint, returning it with the
/// count of handshakes it was asked for.
async fn counted_server(identity: &Arc<Identity>) -> (Endpoint, Arc<AtomicUsize>) {
    let resolver = Arc::new(Resolver::new().expect("resolver"));
    let auth = EndpointAuth::new(Arc::clone(identity), Arc::clone(&resolver));
    let endpoint = bind(&auth, None).await;
    let (protocol, _task) = auth.serve(endpoint.clone()).expect("serve");

    let asked = Arc::new(AtomicUsize::new(0));
    let counted = Counted {
        inner: protocol,
        asked: Arc::clone(&asked),
    };
    let router = Router::builder(endpoint.clone())
        .accept(auth::ALPN, counted)
        .spawn();
    std::mem::forget(router);

    (endpoint, asked)
}

/// A peer that answered and refused is remembered, so a document pointing at
/// it does not cost a handshake on every dial.
#[tokio::test]
async fn a_peer_that_refuses_is_not_asked_again_at_once() {
    let resolver = Arc::new(Resolver::new().expect("resolver"));
    let (server, asked) = counted_server(&unresolvable_identity()).await;

    let client_auth = EndpointAuth::new(identity(), Arc::clone(&resolver));
    let lookup = MemoryLookup::new();
    lookup.add_endpoint_info(server.addr());
    let client = bind(&client_auth, Some(lookup)).await;
    let _client_task = client_auth.serve(client.clone()).expect("serve");

    for _ in 0..2 {
        assert_eq!(
            client_auth
                .prove(server.id(), Duration::from_secs(10))
                .await,
            None
        );
    }

    assert_eq!(
        asked.load(Ordering::Relaxed),
        1,
        "a refusal must be remembered rather than re-handshaked on every request"
    );
}
