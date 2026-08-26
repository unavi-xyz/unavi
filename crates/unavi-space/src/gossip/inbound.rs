use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};

use iroh::Signature;
use iroh_docs::NamespaceId;
use iroh_gossip::api::{
    Event,
    GossipReceiver,
};
use n0_future::StreamExt;
use tokio::sync::Notify;
use tracing::{
    info,
    warn,
};
use unavi_identity::signed_bytes::SignedBytes;

use crate::{
    gossip::{
        GossipCtx,
        SpaceBroadcast,
        SpaceMessage,
    },
    peer::presence::submit_presence,
};

pub async fn handle_gossip_inbound(
    _ctx: &GossipCtx,
    rx: &mut GossipReceiver,
    space: NamespaceId,
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
                let signed_bytes =
                    match postcard::from_bytes::<SignedBytes<SpaceBroadcast>>(&msg.content) {
                        Ok(v) => v,
                        Err(err) => {
                            warn!(?err, "Got invalid gossip message");
                            continue;
                        }
                    };

                let broadcast = match signed_bytes.payload() {
                    Ok(v) => v,
                    Err(err) => {
                        warn!(?err, "Failed to decode gossip payload");
                        continue;
                    }
                };

                let Ok(sig_bytes) = signed_bytes.signature().try_into() else {
                    warn!(
                        "Invalid signature length: {}",
                        signed_bytes.signature().len()
                    );
                    continue;
                };
                let sig = Signature::from_bytes(sig_bytes);

                if let Err(err) = broadcast.sender.verify(&signed_bytes.signing_bytes(), &sig) {
                    warn!(?err, "Invalid gossip signature");
                    continue;
                }

                // TODO create a "disconnect" message variant to clear presence

                match broadcast.msg {
                    SpaceMessage::Presence(peer) => {
                        if peer.id != broadcast.sender {
                            warn!("Presence address does not match sender");
                            continue;
                        }

                        submit_presence(peer, space);
                    }
                    SpaceMessage::Unknown(i) => {
                        warn!("Got unknown gossip variant: {i}");
                    }
                }
            }
        }
    }

    Ok(())
}
