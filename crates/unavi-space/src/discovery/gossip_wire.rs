//! A topic's inbound and outbound gossip: verifying and handing on heard
//! presence, and broadcasting this node's own.

use std::{
    sync::atomic::{
        AtomicUsize,
        Ordering,
    },
    time::Duration,
};

use iroh::Watcher;
use iroh_gossip::api::{
    Event,
    GossipReceiver,
    GossipSender,
};
use n0_future::StreamExt;
use tokio::sync::Notify;
use tracing::{
    debug,
    info,
    warn,
};
use unavi_identity::signed::{
    EndpointSigner,
    Signable,
    Signed,
};

use crate::{
    discovery::{
        PRESENCE_INTERVAL,
        gossip::{
            GossipCtx,
            SpaceBroadcast,
            SpaceMessage,
            unix_secs,
        },
    },
    membership::SpaceId,
};

/// Handles the topic's events until its stream ends.
pub(super) async fn receive(
    ctx: &GossipCtx,
    rx: &mut GossipReceiver,
    space: SpaceId,
    wake: &Notify,
    neighbors: &AtomicUsize,
) -> anyhow::Result<()> {
    while let Some(event) = rx.next().await {
        match event? {
            Event::NeighborUp(n) => {
                info!("+neighbor: {n}");
                neighbors.fetch_add(1, Ordering::Relaxed);
                // Prompt an immediate presence broadcast so the new neighbor is
                // discovered without waiting a full interval. `notify_one`
                // stores a permit, so the wake is not lost while the outbound
                // task is parked.
                wake.notify_one();
            }
            Event::NeighborDown(n) => {
                info!("-neighbor: {n}");
                neighbors
                    .try_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
                        Some(n.saturating_sub(1))
                    })
                    .ok();
            }
            Event::Lagged => warn!("lagged"),
            Event::Received(msg) => {
                let Ok(signed) = postcard::from_bytes::<Signed<SpaceBroadcast>>(&msg.content)
                else {
                    debug!("undecodable gossip message");
                    continue;
                };
                let Ok(broadcast) = signed.payload() else {
                    debug!("undecodable gossip payload");
                    continue;
                };
                if signed.verify_endpoint(broadcast.sender).is_err() {
                    debug!("gossip signature does not match its sender");
                    continue;
                }
                if !broadcast.is_current(space, unix_secs()) {
                    debug!(sender = %broadcast.sender, "stale or misdirected presence");
                    continue;
                }

                match broadcast.msg {
                    SpaceMessage::Presence(addr) => {
                        if addr.id != broadcast.sender {
                            debug!("presence address does not match its sender");
                            continue;
                        }
                        ctx.presence.submit((addr.id, space), addr);
                    }
                }
            }
        }
    }

    Ok(())
}

/// Broadcasts this node's presence while `space` is the active one.
pub(super) async fn announce(
    ctx: &GossipCtx,
    tx: &GossipSender,
    space: SpaceId,
    wake: &Notify,
) -> anyhow::Result<()> {
    let mut active = ctx.active.clone();
    let signer = EndpointSigner(ctx.endpoint.secret_key());
    let mut watcher = ctx.endpoint.watch_addr();

    let _ = n0_future::time::timeout(Duration::from_secs(15), ctx.endpoint.online()).await;

    loop {
        if *active.borrow_and_update() == Some(space) {
            let addr = watcher.get();
            debug!(?addr, "Broadcasting presence");
            let broadcast = SpaceBroadcast {
                sender: ctx.endpoint.id(),
                space,
                issued_at: unix_secs(),
                msg: SpaceMessage::Presence(addr),
            };
            let bytes = postcard::to_stdvec(&broadcast.sign(&signer)?)?;
            tx.broadcast(bytes.into()).await?;
        }

        // Re-broadcast on the interval, immediately when a neighbor joins the
        // topic, or as soon as this space becomes active.
        tokio::select! {
            () = n0_future::time::sleep(PRESENCE_INTERVAL) => {}
            () = wake.notified() => {}
            // The sender outlives every topic task, so a closed channel is
            // unreachable rather than a state to fall through on.
            res = active.changed() => res?,
        }
    }
}
