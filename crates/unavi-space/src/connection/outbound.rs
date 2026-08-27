use std::time::Duration;

use iroh::{
    Endpoint,
    EndpointAddr,
};
use tracing::error;

use crate::{
    connection::{
        ALPN,
        claim_connection,
        release_connection,
    },
    peer::self_peer_id,
};

const MAX_BACKOFF_SECS: u64 = 300;

pub async fn try_open_connection(endpoint: Endpoint, peer: EndpointAddr) {
    let mut delay_secs = 2;

    while let Err(err) = open_connection(endpoint.clone(), peer.clone()).await {
        error!(?err);
        n0_future::time::sleep(Duration::from_secs(delay_secs)).await;
        delay_secs = delay_secs.saturating_mul(2).min(MAX_BACKOFF_SECS);
    }
}

async fn open_connection(endpoint: Endpoint, peer: EndpointAddr) -> anyhow::Result<()> {
    if super::is_blocked(peer.id) {
        return Ok(());
    }

    // The dialer is canonical only if its id is greater.
    let canonical = self_peer_id().is_none_or(|s| s > peer.id);
    let Some((token, cancel_rx)) = claim_connection(peer.id, canonical) else {
        return Ok(());
    };

    let connection = match endpoint.connect(peer.clone(), ALPN).await {
        Ok(connection) => connection,
        Err(err) => {
            release_connection(peer.id, token);
            return Err(err.into());
        }
    };

    let res = super::shared::handle_connection(connection, cancel_rx).await;
    // The peer's replicated state is owned by its inbound stream's `RemotePeer`
    // entity, despawned when the stream ends.
    release_connection(peer.id, token);
    res
}
