//! Dialing tracked peers: one attempt at a time per [`Peer`], backing off on
//! failure, redialing after a connection closes, and giving up after
//! [`MAX_DIAL_FAILURES`] until the peer's address changes.

use std::{
    sync::{
        Arc,
        atomic::{
            AtomicU8,
            Ordering,
        },
    },
    time::Duration,
};

use bevy::prelude::*;
use bevy_async::task;
use bevy_iroh::endpoint::IrohEndpoint;
use iroh::{
    Endpoint,
    EndpointAddr,
};
use tokio::sync::oneshot;

use crate::{
    discovery::Peer,
    link::{
        ALPN,
        PeerLink,
        streams,
    },
};

/// Consecutive failed dials before a peer is left alone.
pub const MAX_DIAL_FAILURES: u32 = 8;

const MIN_BACKOFF: Duration = Duration::from_secs(2);
const MAX_BACKOFF: Duration = Duration::from_mins(5);

/// Wait before redialing a peer whose connection closed cleanly.
const REDIAL_DELAY: Duration = Duration::from_secs(2);

const IDLE: u8 = 0;
const RUNNING: u8 = 1;
const CLOSED: u8 = 2;
const FAILED: u8 = 3;

/// A [`Peer`]'s dial state. Dropping it, with its peer, aborts the attempt
/// and the connection it opened.
#[derive(Component, Default)]
pub struct PeerDial {
    outcome:  Arc<AtomicU8>,
    next:     Duration,
    failures: u32,
    cancel:   Option<oneshot::Sender<()>>,
}

impl PeerDial {
    fn backoff(&self) -> Duration {
        MIN_BACKOFF
            .saturating_mul(1 << self.failures.min(16))
            .min(MAX_BACKOFF)
    }
}

/// Starts a dial for every tracked peer that is not connected and is due.
pub fn dial_peers(
    time: Res<Time>,
    endpoint: Query<&IrohEndpoint>,
    link: Option<Res<PeerLink>>,
    mut peers: Query<(Entity, Ref<Peer>, Option<&mut PeerDial>)>,
    mut commands: Commands,
) {
    let (Ok(endpoint), Some(link)) = (endpoint.single(), link) else {
        return;
    };
    let now = time.elapsed();

    for (entity, peer, dial) in &mut peers {
        let Some(mut dial) = dial else {
            commands.entity(entity).insert(PeerDial::default());
            continue;
        };

        match dial.outcome.load(Ordering::Acquire) {
            RUNNING => continue,
            CLOSED => {
                dial.failures = 0;
                dial.next = now + REDIAL_DELAY;
                dial.outcome.store(IDLE, Ordering::Release);
                dial.cancel = None;
            }
            FAILED => {
                dial.failures += 1;
                dial.next = now + dial.backoff();
                dial.outcome.store(IDLE, Ordering::Release);
                dial.cancel = None;
            }
            _ => {}
        }

        if peer.is_changed() && !peer.is_added() {
            dial.failures = 0;
        }
        if dial.failures >= MAX_DIAL_FAILURES || now < dial.next || link.is_connected(peer.0.id) {
            continue;
        }

        let (cancel_tx, cancel_rx) = oneshot::channel();
        dial.cancel = Some(cancel_tx);
        dial.outcome.store(RUNNING, Ordering::Release);

        let outcome = Arc::clone(&dial.outcome);
        let link = link.clone();
        let endpoint = endpoint.0.clone();
        let addr = peer.0.clone();
        task::spawn(async move {
            tokio::select! {
                _ = cancel_rx => {}
                res = open_connection(&link, endpoint, addr) => {
                    let state = match res {
                        Ok(()) => CLOSED,
                        Err(err) => {
                            debug!(?err, "dial failed");
                            FAILED
                        }
                    };
                    outcome.store(state, Ordering::Release);
                }
            }
        });
    }
}

/// Dials `addr` and runs the connection until it ends.
async fn open_connection(
    link: &PeerLink,
    endpoint: Endpoint,
    addr: EndpointAddr,
) -> anyhow::Result<()> {
    if link.is_blocked(addr.id) {
        return Ok(());
    }

    // The dialer is canonical only if its id is greater.
    let canonical = link.view().me() > addr.id;
    let Some((slot, cancel_rx)) = link.claim_connection(addr.id, canonical) else {
        return Ok(());
    };

    let connection = endpoint.connect(addr, ALPN).await?;
    streams::handle_connection(link, &slot, connection, cancel_rx).await
}
