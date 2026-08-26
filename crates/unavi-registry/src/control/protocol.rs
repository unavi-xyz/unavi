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
    RegistryContext,
    control::{
        RegistryService,
        handle_message,
    },
};

/// Accepts registry calls, pairing each with the DID its connection proved.
///
/// `IrohProtocol::with_sender` would be the shorter path, but it hands the
/// handler a message with no way back to the connection it arrived on, which is
/// the only thing that says who is calling.
pub struct RegistryProtocol {
    ctx: Arc<RegistryContext>,
}

impl std::fmt::Debug for RegistryProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RegistryProtocol")
    }
}

impl RegistryProtocol {
    #[must_use]
    pub const fn new(ctx: Arc<RegistryContext>) -> Self {
        Self { ctx }
    }
}

impl ProtocolHandler for RegistryProtocol {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        // Read once per connection: an endpoint id is fixed for a connection's
        // lifetime, and so is the DID bound to it.
        let caller = self.ctx.bindings.did_of(connection.remote_id());

        while let Some(msg) = irpc_iroh::read_request::<RegistryService>(&connection)
            .await
            .map_err(AcceptError::from_err)?
        {
            let ctx = Arc::clone(&self.ctx);
            let caller = caller.clone();
            n0_future::task::spawn(async move {
                if let Err(err) = handle_message(ctx, caller, msg).await {
                    error!("registry request failed: {err:?}");
                }
            });
        }

        Ok(())
    }
}
