use std::time::Duration;

use iroh::{
    Endpoint,
    EndpointAddr,
};
use tokio::sync::oneshot;
use tracing::error;

use crate::connection::{
    ALPN,
    CONNECTIONS,
};

const MAX_BACKOFF_SECS: u64 = 300;

pub async fn try_open_connection(endpoint: Endpoint, peer: EndpointAddr) {
    let mut delay_secs = 2;

    while let Err(err) = open_connection(endpoint.clone(), peer.clone()).await {
        error!(?err);

        {
            let mut conns = CONNECTIONS.lock().expect("connections lock");
            conns.remove(&peer.id);
        }

        n0_future::time::sleep(Duration::from_secs(delay_secs)).await;
        delay_secs = delay_secs.saturating_mul(2).min(MAX_BACKOFF_SECS);
    }

    let mut conns = CONNECTIONS.lock().expect("connections lock");
    conns.remove(&peer.id);
}

async fn open_connection(endpoint: Endpoint, peer: EndpointAddr) -> anyhow::Result<()> {
    let cancel_rx = {
        let mut conns = CONNECTIONS.lock().expect("connections lock");
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
