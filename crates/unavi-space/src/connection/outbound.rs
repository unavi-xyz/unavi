use std::time::Duration;

use iroh::{Endpoint, EndpointAddr};
use rand::Rng;
use tokio::sync::oneshot;
use tracing::error;

use crate::connection::{ALPN, CONNECTIONS};

pub async fn try_open_connection(endpoint: Endpoint, peer: EndpointAddr) {
    let mut delay_secs = 2;

    while let Err(err) = open_connection(endpoint.clone(), peer.clone()).await {
        error!(?err);

        {
            let mut conns = CONNECTIONS.lock().await;
            conns.remove(&peer.id);
        }

        // If two peers try to connect to each other at the same time, they will
        // only think their own outbound connection attempt is valid and deny the
        // inbound request. (see [`CONNECTIONS`] key tracking)
        //
        // Add random offset, so conflicting peers drift out of sync.
        //
        // TODO This could use a better solution, perhaps a deterministic choosing of
        // one of the two pending connections based on endpoint id.
        let delay_extended = rand::rng().random_range((delay_secs / 2)..(delay_secs * 2));
        n0_future::time::sleep(Duration::from_secs(delay_extended)).await;

        delay_secs = delay_secs.wrapping_mul(2);
    }

    let mut conns = CONNECTIONS.lock().await;
    conns.remove(&peer.id);
}

async fn open_connection(endpoint: Endpoint, peer: EndpointAddr) -> anyhow::Result<()> {
    let cancel_rx = {
        let mut conns = CONNECTIONS.lock().await;
        if conns.contains_key(&peer.id) {
            return Ok(());
        }

        let (cancel_tx, cancel_rx) = oneshot::channel();
        conns.insert(peer.id, cancel_tx);
        cancel_rx
    };

    let connection = endpoint.connect(peer.clone(), ALPN).await?;

    super::shared::handle_connection(connection, cancel_rx).await?;

    Ok(())
}
