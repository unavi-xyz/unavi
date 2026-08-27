use std::time::Duration;

use xdid::{
    core::{
        Method,
        ResolutionError,
        did::Did,
        document::Document,
    },
    methods::{
        key::MethodDidKey,
        web::{
            ClientError,
            Config,
            MethodDidWeb,
        },
    },
    resolver::DidResolver,
};

// TODO(xdid 0.8.1): the did:web client bounds its own target lookup, making
// this redundant. Kept longer than the client's own budget so it cannot
// preempt it.
const RESOLVE_TIMEOUT: Duration = Duration::from_secs(20);

/// Debug builds resolve a `did:web` naming a loopback or private host over
/// plaintext HTTP, so a development server is reachable. Release builds refuse
/// them, where resolving a peer-supplied DID would otherwise probe this host's
/// own network.
const ALLOW_LOCAL: bool = cfg!(debug_assertions);

/// Resolves the DIDs this node verifies against, over `did:key` and `did:web`.
pub struct Resolver(DidResolver);

impl Resolver {
    pub fn new() -> Result<Self, ClientError> {
        let web = MethodDidWeb::with_config(Config {
            allow_local: ALLOW_LOCAL,
            ..Config::default()
        })?;

        let methods: [Box<dyn Method>; 2] = [Box::new(MethodDidKey), Box::new(web)];

        Ok(Self(DidResolver {
            methods: methods.into_iter().collect(),
        }))
    }

    pub async fn resolve(&self, did: &Did) -> Result<Document, ResolutionError> {
        n0_future::time::timeout(RESOLVE_TIMEOUT, self.0.resolve(did))
            .await
            .map_err(|_| ResolutionError::ResolutionFailed("timed out".into()))?
    }
}
