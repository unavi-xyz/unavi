//! The `wired/auth` handshake over two real endpoints: each end proves its DID
//! and binds the one the other proves.
//!
//! The unit tests in `auth::handshake` only exercise the exchange over
//! in-memory streams, so they cannot catch a protocol handler that closes the
//! connection before the peer reads its verdict.

use std::sync::Arc;

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
        EndpointAuth,
    },
    identity::Identity,
    resolver::Resolver,
};
use xdid::method::key::{
    DidKeyPair,
    PublicKey,
    p256::P256KeyPair,
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
