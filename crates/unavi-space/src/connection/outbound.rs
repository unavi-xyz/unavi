use std::time::Duration;

use iroh::{
    Endpoint,
    EndpointAddr,
};
use tracing::error;

use crate::connection::{
    ALPN,
    PeerLink,
};

const MAX_BACKOFF_SECS: u64 = 300;

pub async fn try_open_connection(link: PeerLink, endpoint: Endpoint, peer: EndpointAddr) {
    let mut delay_secs = 2;

    while let Err(err) = open_connection(&link, endpoint.clone(), peer.clone()).await {
        error!(?err);
        n0_future::time::sleep(Duration::from_secs(delay_secs)).await;
        delay_secs = delay_secs.saturating_mul(2).min(MAX_BACKOFF_SECS);
    }
}

async fn open_connection(
    link: &PeerLink,
    endpoint: Endpoint,
    peer: EndpointAddr,
) -> anyhow::Result<()> {
    if link.is_blocked(peer.id) {
        return Ok(());
    }

    // The dialer is canonical only if its id is greater.
    let canonical = link.view().me() > peer.id;
    let Some((token, cancel_rx)) = link.claim_connection(peer.id, canonical) else {
        return Ok(());
    };

    let connection = match endpoint.connect(peer.clone(), ALPN).await {
        Ok(connection) => connection,
        Err(err) => {
            link.release_connection(peer.id, token);
            return Err(err.into());
        }
    };

    let res = super::shared::handle_connection(link, connection, cancel_rx).await;
    // The peer's replicated state is owned by its inbound stream's `RemotePeer`
    // entity, despawned when the stream ends.
    link.release_connection(peer.id, token);
    res
}
