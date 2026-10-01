//! The `wired/registry` protocol handler.

use std::sync::Arc;

use iroh::{
    endpoint::Connection,
    protocol::{
        AcceptError,
        ProtocolHandler,
    },
};
use tracing::error;

use crate::{
    rpc::RegistryService,
    server::{
        Shared,
        handlers::handle_message,
    },
};

/// Accepts registry calls, pairing each with the endpoint it arrived from.
///
/// `IrohProtocol::with_sender` would be the shorter path, but it hands the
/// handler a message with no way back to the connection it arrived on, which is
/// the only thing that says who is calling.
pub struct RegistryProtocol {
    shared: Arc<Shared>,
}

impl std::fmt::Debug for RegistryProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RegistryProtocol")
    }
}

impl RegistryProtocol {
    pub(crate) const fn new(shared: Arc<Shared>) -> Self {
        Self { shared }
    }
}

impl ProtocolHandler for RegistryProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let remote = connection.remote_id();

        while let Some(msg) = irpc_iroh::read_request::<RegistryService>(&connection)
            .await
            .map_err(AcceptError::from_err)?
        {
            let shared = Arc::clone(&self.shared);
            n0_future::task::spawn(async move {
                if let Err(err) = handle_message(shared, remote, msg).await {
                    error!("registry request failed: {err:?}");
                }
            });
        }

        Ok(())
    }
}
