use std::{sync::Arc, time::Duration};

use bevy::prelude::*;
use bevy_iroh::endpoint::IrohEndpoint;
use iroh::{Endpoint, EndpointAddr};
use tokio::sync::Notify;
use tracing::Instrument;
use unavi_util::async_task::spawn_async_task;

use crate::Peer;

mod protocol;

pub fn connect_to_peer(
    trigger: On<Add, Peer>,
    peers: Query<&Peer>,
    endpoint: Query<&IrohEndpoint>,
    mut commands: Commands,
) {
    let Ok(endpoint) = endpoint.single().map(|e| e.0.clone()) else {
        warn!("Unable to connect to peer: no endpoint");
        return;
    };

    let peer = peers
        .get(trigger.entity)
        .map(|p| p.0.clone())
        .expect("peer");

    let cancel = Arc::new(Notify::default());
    commands
        .entity(trigger.entity)
        .insert(PeerCancel(Arc::clone(&cancel)));

    let span = info_span!("connect", peer = %peer.id);
    spawn_async_task(
        async move {
            let mut delay_secs = 2;

            while let Err(err) = inner(endpoint.clone(), peer.clone(), &cancel).await {
                error!(?err);
                n0_future::time::sleep(Duration::from_secs(delay_secs)).await;
                delay_secs = delay_secs.wrapping_mul(2);
            }
        }
        .instrument(span),
    );
}

#[derive(Component)]
pub struct PeerCancel(Arc<Notify>);

impl Drop for PeerCancel {
    fn drop(&mut self) {
        self.0.notify_waiters();
        self.0.notify_one();
    }
}

pub fn disconnect_peer(trigger: On<Remove, Peer>, mut commands: Commands) {
    commands.entity(trigger.entity).remove::<PeerCancel>();
}

async fn inner(endpoint: Endpoint, peer: EndpointAddr, cancel: &Arc<Notify>) -> anyhow::Result<()> {
    tokio::select! {
        () = cancel.notified() => Ok(()),
        res = open_connection(endpoint, peer) => res,
    }
}

async fn open_connection(endpoint: Endpoint, peer: EndpointAddr) -> anyhow::Result<()> {
    let conn = endpoint.connect(peer, protocol::ALPN).await?;
    info!("Connected");

    let (tx, rx) = conn.open_bi().await?;

    Ok(())
}
