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

/// Intercepts connection establishment so a peer is identified before the
/// protocol that wanted it runs.
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

        // Accepted whether or not the proof succeeds: reads are open to
        // anyone, so a peer that answers nothing is anonymous rather than
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
        _conn: &'a Connection,
    ) -> impl Future<Output = AfterHandshakeOutcome> + Send + 'a {
        std::future::ready(AfterHandshakeOutcome::accept())
    }
}
