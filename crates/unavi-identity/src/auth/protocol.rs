//! The accepting side of `wired/auth`.

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

use crate::{
    auth::{
        CLOSE_REFUSED,
        PROOF_DEADLINE,
        bindings::Bindings,
        handshake::Handshake,
    },
    identity::Identity,
    resolver::Resolver,
};

/// Answers `wired/auth`. Registered on the router under [`crate::auth::ALPN`].
#[derive(Clone, Debug)]
pub struct AuthProtocol {
    local:    EndpointId,
    bindings: Arc<Bindings>,
    identity: Arc<Identity>,
    resolver: Arc<Resolver>,
}

impl AuthProtocol {
    pub(crate) const fn new(
        local: EndpointId,
        bindings: Arc<Bindings>,
        identity: Arc<Identity>,
        resolver: Arc<Resolver>,
    ) -> Self {
        Self {
            local,
            bindings,
            identity,
            resolver,
        }
    }
}

impl ProtocolHandler for AuthProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let remote = connection.remote_id();

        let handshake = Handshake {
            identity: &self.identity,
            resolver: &self.resolver,
            local: self.local,
            remote,
        };

        let exchange = async {
            let (mut tx, mut rx) = connection.accept_bi().await?;
            handshake
                .accept(&mut tx, &mut rx, |did| {
                    self.bindings.bind(remote, did.clone());
                })
                .await
        };

        match n0_future::time::timeout(PROOF_DEADLINE, exchange).await {
            Ok(Ok(did)) => debug!(%did, "peer identified"),
            Ok(Err(err)) => {
                debug!(?err, "peer proved no identity");
                connection.close(VarInt::from_u32(CLOSE_REFUSED), b"proof refused");
            }
            Err(_) => {
                debug!(%remote, "identity proof timed out");
                connection.close(VarInt::from_u32(CLOSE_REFUSED), b"proof timed out");
            }
        }

        Ok(())
    }
}
