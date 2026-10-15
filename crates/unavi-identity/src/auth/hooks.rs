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

use crate::auth::{
    ALPN,
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
        // refused. Awaited so the dial behind it sees the binding.
        let (tx, rx) = oneshot::channel();
        if self
            .tx
            .send(Message::Prove(remote_addr.id, tx))
            .await
            .is_ok()
        {
            rx.await.ok();
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
