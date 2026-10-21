//! Endpoint hooks: prove ahead of every dial, and track each peer's
//! connections so its binding ends with them.

use std::sync::Arc;

use iroh::{
    EndpointAddr,
    endpoint::{
        AfterHandshakeOutcome,
        BeforeConnectOutcome,
        Connection,
        EndpointHooks,
    },
};
use tokio::sync::{
    mpsc,
    oneshot,
};
use tracing::debug;

use crate::auth::{
    ALPN,
    BIND_WAIT,
    Message,
    bindings::Bindings,
};

#[derive(Debug)]
pub struct Hooks {
    pub tx:       mpsc::Sender<Message>,
    pub bindings: Arc<Bindings>,
}

impl EndpointHooks for Hooks {
    async fn before_connect<'a>(
        &'a self,
        remote_addr: &'a EndpointAddr,
        alpn: &'a [u8],
    ) -> BeforeConnectOutcome {
        if alpn == ALPN || self.bindings.is_bound(remote_addr.id) {
            return BeforeConnectOutcome::Accept;
        }

        // Accepted whether or not the proof succeeds. Reads are open to
        // anyone, so a peer that proves nothing is anonymous rather than
        // refused. Waited on only briefly: the first dial to a cold peer
        // spends its time on relay and holepunch discovery, and holding the
        // caller there postpones the very connection the proof runs on. A
        // caller that needs the DID asks for one instead of reading the
        // binding.
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(Message::Prove(remote_addr.id, tx))
            .await
            .is_ok()
            && n0_future::time::timeout(BIND_WAIT, rx).await.is_err()
        {
            debug!(remote = %remote_addr.id.fmt_short(), "dial went ahead unproven");
        }

        BeforeConnectOutcome::Accept
    }

    fn after_handshake<'a>(
        &'a self,
        conn: &'a Connection,
    ) -> impl Future<Output = AfterHandshakeOutcome> + Send + 'a {
        let peer = conn.remote_id();
        // Registered now, while this call still holds the connection, so the
        // close is seen however soon it comes.
        let closed = conn.weak_handle().closed();
        self.bindings.connection_opened(peer);

        let bindings = Arc::clone(&self.bindings);
        n0_future::task::spawn(async move {
            closed.await;
            bindings.connection_closed(peer);
        });

        std::future::ready(AfterHandshakeOutcome::accept())
    }
}
