use xdid::{
    core::{
        Method,
        ResolutionError,
        did::Did,
        document::Document,
    },
    method::{
        key::MethodDidKey,
        web::{
            ClientError,
            Config,
            MethodDidWeb,
            target::TargetPolicy,
        },
    },
    resolver::DidResolver,
};

/// Debug builds resolve a `did:web` naming a loopback or private host over
/// plaintext HTTP, so a development server is reachable. Release builds refuse
/// them, where resolving a peer-supplied DID would otherwise probe this host's
/// own network.
const TARGET: TargetPolicy = if cfg!(debug_assertions) {
    TargetPolicy::AllowLocal
} else {
    TargetPolicy::PublicOnly
};

/// Resolves the DIDs this node verifies against, over `did:key` and `did:web`.
pub struct Resolver(DidResolver);

impl Resolver {
    pub fn new() -> Result<Self, ClientError> {
        let web = MethodDidWeb::with_config(Config {
            target: TARGET,
            ..Config::default()
        })?;

        Ok(Self(DidResolver::with_methods([
            Box::new(MethodDidKey) as Box<dyn Method>,
            Box::new(web),
        ])))
    }

    pub async fn resolve(&self, did: &Did) -> Result<Document, ResolutionError> {
        self.0.resolve(did).await
    }
}
