//! The `wired/registry` protocol handler.

use std::{
    sync::Arc,
    time::Duration,
};

use iroh::{
    endpoint::Connection,
    protocol::{
        AcceptError,
        ProtocolHandler,
    },
};
use tracing::warn;

use crate::{
    rpc::RegistryService,
    server::{
        Shared,
        handlers::handle_message,
    },
};

/// Longest one request may take, so a slow identity resolution cannot hold a
/// connection forever.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Accepts registry calls, pairing each with the endpoint it arrived from.
/// A connection's requests run one at a time, so one peer cannot multiply its
/// concurrent identity resolutions.
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
            let handled = n0_future::time::timeout(
                REQUEST_TIMEOUT,
                handle_message(Arc::clone(&self.shared), remote, msg),
            )
            .await;
            match handled {
                Ok(Ok(())) => {}
                Ok(Err(err)) => warn!(%remote, "registry request failed: {err:?}"),
                Err(_) => warn!(%remote, "registry request timed out"),
            }
        }

        Ok(())
    }
}
