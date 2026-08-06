//! Shared, time-bounded DID resolution.

use std::{
    sync::LazyLock,
    time::Duration,
};

use xdid::{
    core::{
        Method,
        did::Did,
        document::Document,
    },
    methods::{
        key::MethodDidKey,
        web::{
            Config,
            MethodDidWeb,
        },
    },
    resolver::DidResolver,
};

/// Bounds the whole resolution. The `did:web` client caps its own connect and
/// request phases, but the target check ahead of them resolves DNS without a
/// deadline of its own.
const RESOLVE_TIMEOUT: Duration = Duration::from_secs(5);

/// Built once: each `DidResolver` carries its own HTTP connection pool, so
/// rebuilding it per request would discard every kept-alive connection.
static STRICT: LazyLock<Option<DidResolver>> = LazyLock::new(|| build(false));

static LOOPBACK: LazyLock<Option<DidResolver>> = LazyLock::new(|| build(true));

fn build(allow_local: bool) -> Option<DidResolver> {
    let config = Config {
        allow_local,
        ..Config::default()
    };

    let methods: [Box<dyn Method>; 2] = [
        Box::new(MethodDidKey),
        Box::new(MethodDidWeb::with_config(config).ok()?),
    ];

    Some(DidResolver {
        methods: methods.into_iter().collect(),
    })
}

/// Resolves `did`, or returns `None` if resolution timed out or failed.
///
/// Refuses `did:web` targets outside public unicast space. Every path reachable
/// by a remote peer must resolve through this, since a DID chosen by that peer
/// otherwise names the outbound request's destination.
pub async fn resolve(did: &Did) -> Option<Document> {
    run(STRICT.as_ref()?, did).await
}

/// Resolves `did`, additionally permitting loopback and private targets over
/// plaintext HTTP.
///
/// Only for DIDs an operator configured, never for one carried in a request: it
/// turns resolution into a probe of whatever the host can reach.
pub async fn resolve_allowing_loopback(did: &Did) -> Option<Document> {
    run(LOOPBACK.as_ref()?, did).await
}

async fn run(resolver: &DidResolver, did: &Did) -> Option<Document> {
    match n0_future::time::timeout(RESOLVE_TIMEOUT, resolver.resolve(did)).await {
        Ok(Ok(doc)) => Some(doc),
        Ok(Err(err)) => {
            tracing::debug!(%did, "did resolution failed: {err}");
            None
        }
        Err(_) => {
            tracing::warn!(%did, "did resolution timed out");
            None
        }
    }
}
