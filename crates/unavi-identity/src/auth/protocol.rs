use std::sync::Arc;

use iroh::{
    EndpointId,
    endpoint::{
        Connection,
        VarInt,
    },
    protocol::{
        AcceptError,
        ProtocolHandler,
    },
};
use tracing::debug;
use xdid::resolver::DidResolver;

use crate::{
    auth::{
        CLOSE_REFUSED,
        bindings::Bindings,
        handshake::Handshake,
    },
    identity::Identity,
};

/// Answers `wired/auth`. Registered on the router under [`crate::auth::ALPN`].
#[derive(Clone)]
pub struct Protocol {
    pub local:    EndpointId,
    pub bindings: Arc<Bindings>,
    pub identity: Arc<Identity>,
    pub resolver: Arc<DidResolver>,
}

/// `DidResolver` holds trait objects, so a derived `Debug` cannot cover it.
impl std::fmt::Debug for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Protocol")
            .field("local", &self.local)
            .field("bindings", &self.bindings)
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}

impl ProtocolHandler for Protocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let remote = connection.remote_id();
        let (mut tx, mut rx) = connection.accept_bi().await?;

        let handshake = Handshake {
            identity: &self.identity,
            resolver: &self.resolver,
            local: self.local,
            remote,
        };

        match handshake.accept(&mut tx, &mut rx).await {
            Ok(did) => {
                debug!(%did, "peer identified");
                self.bindings.bind(remote, did);
            }
            Err(err) => {
                debug!(?err, "peer proved no identity");
                connection.close(VarInt::from_u32(CLOSE_REFUSED), b"proof refused");
            }
        }

        Ok(())
    }
}
