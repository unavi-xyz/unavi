use std::{sync::Arc, time::Duration};

use iroh::{Endpoint, EndpointAddr};
use rand::Rng;
use tokio::sync::Notify;
use tracing::error;

use crate::connection::CONNECTIONS;

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
        // inbound request.
        //
        // Add random offset, so conflicting peers drift out of sync.
        let delay_extended = delay_secs + rand::rng().random_range(0..delay_secs);
        n0_future::time::sleep(Duration::from_secs(delay_extended)).await;

        delay_secs = delay_secs.wrapping_mul(2);
    }

    let mut conns = CONNECTIONS.lock().await;
    conns.remove(&peer.id);
}

async fn open_connection(endpoint: Endpoint, peer: EndpointAddr) -> anyhow::Result<()> {
    let cancel = {
        let mut conns = CONNECTIONS.lock().await;
        if conns.contains_key(&peer.id) {
            return Ok(());
        }

        let cancel = Arc::new(Notify::default());
        conns.insert(peer.id, Arc::clone(&cancel));
        cancel
    };

    let connection = tokio::select! {
        () = cancel.notified() => return Ok(()),
        res = endpoint.connect(peer, super::ALPN) => res?,
    };

    super::shared::handle_connection(&connection, &cancel).await?;

    Ok(())
}
