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

/// Which hosts a resolution may reach.
#[derive(Clone, Copy)]
pub enum Space {
    Public,
    /// Also loopback and private targets, over plaintext HTTP. Only for a DID
    /// an operator configured; a peer-supplied one turns resolution into a
    /// probe of this host.
    Local,
}

// TODO(xdid 0.8.1): the did:web client bounds its own target lookup, making
// this redundant. Kept longer than the client's own budget so it cannot
// preempt it.
const RESOLVE_TIMEOUT: Duration = Duration::from_secs(20);

pub struct Resolver {
    public: DidResolver,
    local:  DidResolver,
}

impl Resolver {
    pub fn new() -> Result<Self, ClientError> {
        Ok(Self {
            public: methods(false)?,
            local:  methods(true)?,
        })
    }

    pub async fn resolve(&self, did: &Did, space: Space) -> Result<Document, ResolutionError> {
        let resolver = match space {
            Space::Public => &self.public,
            Space::Local => &self.local,
        };

        n0_future::time::timeout(RESOLVE_TIMEOUT, resolver.resolve(did))
            .await
            .map_err(|_| ResolutionError::ResolutionFailed("timed out".into()))?
    }
}

fn methods(allow_local: bool) -> Result<DidResolver, ClientError> {
    let web = MethodDidWeb::with_config(Config {
        allow_local,
        ..Config::default()
    })?;

    let methods: [Box<dyn Method>; 2] = [Box::new(MethodDidKey), Box::new(web)];

    Ok(DidResolver {
        methods: methods.into_iter().collect(),
    })
}
